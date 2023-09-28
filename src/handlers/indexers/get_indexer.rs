use axum::extract::State;
use axum::Json;
use uuid::Uuid;

use crate::domain::models::indexer::{IndexerError, IndexerModel};
use crate::infra::repositories::indexer_repository::{IndexerRepository, Repository};
use crate::utils::PathExtractor;
use crate::AppState;

pub async fn get_indexer(
    State(state): State<AppState>,
    PathExtractor(id): PathExtractor<Uuid>,
) -> Result<Json<IndexerModel>, IndexerError> {
    let repository = IndexerRepository::new(&state.pool);
    let indexer_model = repository.get(id).await.map_err(IndexerError::InfraError)?;

    Ok(Json(indexer_model))
}
