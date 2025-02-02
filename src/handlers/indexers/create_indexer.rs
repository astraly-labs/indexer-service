use std::net::TcpListener;
use std::str::FromStr;

use axum::body::Bytes;
use axum::extract::{Multipart, State};
use axum::Json;
use diesel::SelectableHelper;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::Deserialize;
use uuid::Uuid;

use super::fail_indexer::fail_indexer;
use super::start_indexer::start_indexer;
use super::utils::query_status_server;
use crate::config::config;
use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerStatus, IndexerType};
use crate::handlers::indexers::utils::get_s3_script_key;
use crate::infra::db::schema::indexers;
use crate::infra::errors::InfraError;
use crate::infra::repositories::indexer_repository::{self, IndexerDb};
use crate::utils::env::get_environment_variable;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateIndexerRequest {
    pub indexer_type: IndexerType,
    pub target_url: Option<String>,
    pub table_name: Option<String>,
    pub custom_connection_string: Option<String>,
    pub starting_block: Option<i64>,
    pub indexer_id: Option<String>,
    #[serde(skip)]
    pub data: Bytes,
    #[serde(skip)]
    pub status_server_port: i32,
}

impl Default for CreateIndexerRequest {
    fn default() -> Self {
        Self {
            indexer_type: IndexerType::default(),
            target_url: None,
            table_name: None,
            custom_connection_string: None,
            starting_block: None,
            indexer_id: None,
            data: Bytes::new(),
            status_server_port: 1234,
        }
    }
}

impl CreateIndexerRequest {
    /// Check if the request is ready to be processed
    fn is_ready(&self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        match self.indexer_type {
            IndexerType::Postgres => {
                if self.table_name.is_none() {
                    return false;
                }
            }
            IndexerType::Webhook => {
                if self.target_url.is_none() {
                    return false;
                }
            }
        };
        true
    }

    /// Set a random available port for the gRPC status server
    fn set_random_port(&mut self) {
        // Bind to a random port
        if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
            if let Ok(addr) = listener.local_addr() {
                // Set the found port
                self.status_server_port = addr.port() as i32;
            }
        }
    }
}

// not using From trait as we need async functions
async fn build_create_indexer_request(request: &mut Multipart) -> Result<CreateIndexerRequest, IndexerError> {
    let mut create_indexer_request = CreateIndexerRequest::default();
    while let Some(field) = request.next_field().await.map_err(IndexerError::FailedToReadMultipartField)? {
        let field_name = field.name().ok_or(IndexerError::InternalServerError("Failed to get field name".into()))?;
        match field_name {
            "script.js" => {
                create_indexer_request.data = field.bytes().await.map_err(IndexerError::FailedToReadMultipartField)?
            }
            "target_url" => {
                create_indexer_request.target_url =
                    Some(field.text().await.map_err(IndexerError::FailedToReadMultipartField)?)
            }
            "table_name" => {
                create_indexer_request.table_name =
                    Some(field.text().await.map_err(IndexerError::FailedToReadMultipartField)?)
            }
            "indexer_type" => {
                let field = field.text().await.map_err(IndexerError::FailedToReadMultipartField)?;
                create_indexer_request.indexer_type =
                    IndexerType::from_str(field.as_str()).map_err(|_| IndexerError::InvalidIndexerType(field))?
            }
            "starting_block" => {
                let field = field.text().await.map_err(IndexerError::FailedToReadMultipartField)?;
                create_indexer_request.starting_block = Some(
                    field.parse().map_err(|_| IndexerError::InternalServerError("Invalid starting block".into()))?,
                );
            }
            "indexer_id" => {
                create_indexer_request.indexer_id =
                    Some(field.text().await.map_err(IndexerError::FailedToReadMultipartField)?)
            }
            _ => return Err(IndexerError::UnexpectedMultipartField(field_name.to_string())),
        };
    }

    create_indexer_request.set_random_port();

    // For Postgres indexers, use table_name as indexer_id if not provided
    if create_indexer_request.indexer_type == IndexerType::Postgres && create_indexer_request.indexer_id.is_none() {
        create_indexer_request.indexer_id = create_indexer_request.table_name.clone();
    }

    if !create_indexer_request.is_ready() {
        return Err(IndexerError::FailedToBuildCreateIndexerRequest);
    }

    Ok(create_indexer_request)
}

pub async fn create_indexer(
    State(state): State<AppState>,
    mut request: Multipart,
) -> Result<Json<IndexerModel>, IndexerError> {
    let id = Uuid::new_v4();
    let create_indexer_request = build_create_indexer_request(&mut request).await?;
    let new_indexer_db = indexer_repository::NewIndexerDb {
        id,
        status: IndexerStatus::Created.to_string(),
        type_: create_indexer_request.indexer_type.to_string(),
        target_url: create_indexer_request.target_url.clone(),
        table_name: create_indexer_request.table_name.clone(),
        status_server_port: None,
        custom_connection_string: create_indexer_request.custom_connection_string.clone(),
        starting_block: create_indexer_request.starting_block,
        indexer_id: create_indexer_request.indexer_id.clone(),
    };

    let config = config().await;
    let bucket_name = get_environment_variable("INDEXER_SERVICE_BUCKET");

    let connection = &mut state.pool.get().await.expect("Failed to get a connection from the pool");
    let created_indexer = connection
        .transaction::<_, IndexerError, _>(|conn| {
            async move {
                let created_indexer: IndexerModel = diesel::insert_into(indexers::table)
                    .values(new_indexer_db)
                    .returning(IndexerDb::as_returning())
                    .get_result::<IndexerDb>(conn)
                    .await?
                    .try_into()
                    .map_err(|e| IndexerError::InfraError(InfraError::ParseError(e)))?;

                config
                    .s3_client()
                    .put_object()
                    .bucket(bucket_name)
                    .key(get_s3_script_key(id))
                    .body(create_indexer_request.data.into())
                    .send()
                    .await
                    .map_err(IndexerError::FailedToUploadToS3)?;

                Ok(created_indexer)
            }
            .scope_boxed()
        })
        .await?;

    start_indexer(created_indexer.id).await?;

    // wait a bit for the indexer to start
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // check the status server from apibara
    // if we have an error we simply return the error and shutdown the indexer
    let server_port = created_indexer.status_server_port.ok_or(IndexerError::IndexerStatusServerPortNotFound)?;
    let server_status = query_status_server(server_port).await?;
    if server_status.status != 1 {
        fail_indexer(created_indexer.id).await?;
    }

    Ok(Json(created_indexer))
}
