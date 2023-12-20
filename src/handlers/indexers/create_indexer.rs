use std::str::FromStr;

use axum::body::Bytes;
use axum::extract::{Multipart, State};
use axum::Json;
use diesel::SelectableHelper;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::config::config;
use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerStatus, IndexerType};
use crate::handlers::indexers::utils::get_s3_script_key;
use crate::infra::db::schema::indexers;
use crate::infra::errors::InfraError;
use crate::infra::repositories::indexer_repository::{self, IndexerDb};
use crate::publishers::indexers::publish_start_indexer;
use crate::utils::env::get_environment_variable;
use crate::AppState;

#[derive(Default)]
struct CreateIndexerRequest {
    target_url: Option<String>,
    data: Bytes,
    table_name: Option<String>,
    indexer_type: IndexerType,
}

impl CreateIndexerRequest {
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
            _ => return Err(IndexerError::UnexpectedMultipartField(field_name.to_string())),
        };
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
        status: IndexerStatus::Created.to_string(),
        indexer_type: create_indexer_request.indexer_type.to_string(),
        id,
        target_url: create_indexer_request.target_url,
        table_name: create_indexer_request.table_name,
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

    publish_start_indexer(id, 1, 0).await?;

    Ok(Json(created_indexer))
}
