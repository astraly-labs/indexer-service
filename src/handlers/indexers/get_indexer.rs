use axum::extract::State;
use axum::Json;
use uuid::Uuid;

use super::utils::query_status_server;
use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerServerStatus};
use crate::infra::repositories::indexer_repository::{IndexerFilter, IndexerRepository, Repository};
use crate::utils::PathExtractor;
use crate::AppState;

pub async fn get_indexers(State(state): State<AppState>) -> Result<Json<Vec<IndexerModel>>, IndexerError> {
    let repository = IndexerRepository::new(&state.pool);
    let indexers = repository.get_all(IndexerFilter { status: None }).await.map_err(IndexerError::InfraError)?;

    Ok(Json(indexers))
}

pub async fn get_indexer(
    State(state): State<AppState>,
    PathExtractor(id): PathExtractor<Uuid>,
) -> Result<Json<IndexerModel>, IndexerError> {
    let repository = IndexerRepository::new(&state.pool);
    let indexer_model = repository.get(id).await.map_err(IndexerError::InfraError)?;

    Ok(Json(indexer_model))
}

pub async fn get_indexer_status(
    State(state): State<AppState>,
    PathExtractor(id): PathExtractor<Uuid>,
) -> Result<Json<IndexerServerStatus>, IndexerError> {
    let repository = IndexerRepository::new(&state.pool);
    let indexer_model = repository.get(id).await.map_err(IndexerError::InfraError)?;

    let server_port = indexer_model.status_server_port.ok_or(IndexerError::IndexerStatusServerPortNotFound)?;

    let status_response = query_status_server(server_port).await?;

    Ok(Json(status_response))
}

pub async fn get_indexer_status_by_table_name(
    State(state): State<AppState>,
    PathExtractor(table_name): PathExtractor<String>,
) -> Result<Json<IndexerServerStatus>, IndexerError> {
    let repository = IndexerRepository::new(&state.pool);
    let indexer_model = repository.get_by_table_name(table_name).await.map_err(IndexerError::InfraError)?;

    let server_port = indexer_model.status_server_port.ok_or(IndexerError::IndexerStatusServerPortNotFound)?;

    let status_response = query_status_server(server_port).await?;

    Ok(Json(status_response))
}
