use axum::extract::State;
use uuid::Uuid;

use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::infra::repositories::indexer_repository::{IndexerRepository, Repository};
use crate::utils::PathExtractor;
use crate::AppState;

pub async fn delete_indexer(
    State(state): State<AppState>,
    PathExtractor(id): PathExtractor<Uuid>,
) -> Result<(), IndexerError> {
    let mut repository = IndexerRepository::new(&state.pool);
    let indexer_model = repository.get(id).await.map_err(IndexerError::InfraError)?;
    match indexer_model.status {
        IndexerStatus::Stopped => (),
        _ => return Err(IndexerError::InvalidIndexerStatus(indexer_model.status)),
    }

    repository.delete(id).await.map_err(IndexerError::InfraError)?;

    Ok(())
}
