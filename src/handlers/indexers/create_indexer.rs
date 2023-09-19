use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerStatus, IndexerType};
use crate::domain::models::post::PostError;
use crate::handlers::posts::{CreatePostRequest, PostResponse};
use crate::infra::repositories::{indexer_repository, post_repository};
use crate::publishers::indexers::publish_start_indexer;
use crate::utils::JsonExtractor;
use crate::AppState;
use axum::body::HttpBody;
use axum::extract::{Multipart, State};
use axum::Json;
use std::fs;
use std::io::Write;
use uuid::Uuid;

pub async fn create_indexer(
    State(state): State<AppState>,
    mut request: Multipart,
) -> Result<Json<IndexerModel>, IndexerError> {
    let id = Uuid::new_v4();
    let new_indexer_db = indexer_repository::NewIndexerDb {
        status: IndexerStatus::Created.to_string(),
        indexer_type: IndexerType::Webhook.to_string(),
        id: id.clone(),
    };

    if let Some(file) = request.next_field().await.map_err(IndexerError::FailedToReadFile)? {
        let filename = file.name().ok_or(IndexerError::InternalServerError)?;
        if filename != "script.js" {
            return Err(IndexerError::IncorrectFileName);
        }
        let data = file.bytes().await.map_err(IndexerError::FailedToReadFile)?;
        let mut file = fs::File::create(format!("{}/{}/{}.js", env!("CARGO_MANIFEST_DIR"), "scripts", id.to_string()))
            .map_err(IndexerError::FailedToCreateFile)?;
        file.write_all(data.to_vec().as_slice()).map_err(IndexerError::FailedToCreateFile)?;
    }

    let created_indexer =
        indexer_repository::insert(&state.pool, new_indexer_db).await.map_err(IndexerError::InfraError)?;

    publish_start_indexer(id).await.map_err(IndexerError::FailedToPushToQueue)?;

    Ok(Json(created_indexer))
}
