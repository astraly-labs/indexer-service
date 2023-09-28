use axum::extract::{Multipart, State};
use axum::Json;
use diesel::SelectableHelper;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::config::config;
use crate::constants::s3::INDEXER_SERVICE_BUCKET;
use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerStatus, IndexerType};
use crate::handlers::indexers::utils::get_s3_script_key;
use crate::infra::db::schema::indexers;
use crate::infra::repositories::indexer_repository::{self, IndexerDb};
use crate::publishers::indexers::publish_start_indexer;
use crate::AppState;

pub async fn create_indexer(
    State(state): State<AppState>,
    mut request: Multipart,
) -> Result<Json<IndexerModel>, IndexerError> {
    let id = Uuid::new_v4();
    let new_indexer_db = indexer_repository::NewIndexerDb {
        status: IndexerStatus::Created.to_string(),
        indexer_type: IndexerType::Webhook.to_string(),
        id,
    };

    if let Some(file) = request.next_field().await.map_err(IndexerError::FailedToReadFile)? {
        let filename = file.name().ok_or(IndexerError::InternalServerError)?;
        if filename != "script.js" {
            return Err(IndexerError::IncorrectFileName);
        }
        let data = file.bytes().await.map_err(IndexerError::FailedToReadFile)?;

        let config = config().await;

        let connection = &mut state.pool.get().await.expect("Failed to get a connection from the pool");
        let created_indexer = connection
            .transaction::<_, IndexerError, _>(|conn| {
                async move {
                    let created_indexer: IndexerModel = diesel::insert_into(indexers::table)
                        .values(new_indexer_db)
                        .returning(IndexerDb::as_returning())
                        .get_result(conn)
                        .await?
                        .into();

                    config
                        .s3_client()
                        .put_object()
                        .bucket(INDEXER_SERVICE_BUCKET)
                        .key(get_s3_script_key(id))
                        .body(data.into())
                        .send()
                        .await
                        .map_err(IndexerError::FailedToUploadToS3)?;

                    Ok(created_indexer)
                }
                .scope_boxed()
            })
            .await?;

        publish_start_indexer(id).await.map_err(IndexerError::FailedToPushToQueue)?;

        return Ok(Json(created_indexer));
    }

    Err(IndexerError::NoFileFound)
}
