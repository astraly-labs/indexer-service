use axum::extract::State;
use uuid::Uuid;

use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::handlers::indexers::indexer_types::{get_indexer_handler, Indexer};
use crate::infra::repositories::indexer_repository;
use crate::infra::repositories::indexer_repository::UpdateIndexerStatusDb;
use crate::utils::PathExtractor;
use crate::AppState;

pub async fn stop_indexer(
    State(state): State<AppState>,
    PathExtractor(id): PathExtractor<Uuid>,
) -> Result<(), IndexerError> {
    let indexer_model = indexer_repository::get(&state.pool, id).await.map_err(IndexerError::InfraError)?;
    match indexer_model.status {
        IndexerStatus::Running => (),
        _ => return Err(IndexerError::InvalidIndexerStatus(indexer_model.status)),
    }

    let indexer = get_indexer_handler(&indexer_model.indexer_type);

    // TODO: check if command failed because indexer was already stopped, in that case update status to
    // Stopped
    let new_status = match indexer.stop(indexer_model).await {
        Ok(_) => IndexerStatus::Stopped,
        Err(_) => IndexerStatus::FailedStopping,
    };

    indexer_repository::update_status(&state.pool, UpdateIndexerStatusDb { id, status: new_status.to_string() })
        .await
        .map_err(IndexerError::InfraError)?;

    Ok(())
}
