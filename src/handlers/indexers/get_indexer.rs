use axum::extract::State;
use axum::Json;
use uuid::Uuid;

use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerServerStatus};
use crate::grpc::apibara_sink_v1::status_client::StatusClient;
use crate::grpc::apibara_sink_v1::GetStatusRequest;
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

    // Create a gRPC client
    let endpoint = format!("http://localhost:{}", server_port);
    let mut client = StatusClient::connect(endpoint).await.map_err(IndexerError::FailedToConnectGRPC)?;

    // Create a GetStatusRequest
    let request = tonic::Request::new(GetStatusRequest {});

    // Fetch the status
    let response = client.get_status(request).await.map_err(IndexerError::GRPCRequestFailed)?;

    // Process the response
    let status_response = response.into_inner();

    Ok(Json(status_response.into()))
}

pub async fn get_indexer_status_by_table_name(
    State(state): State<AppState>,
    PathExtractor(table_name): PathExtractor<String>,
) -> Result<Json<IndexerServerStatus>, IndexerError> {
    let repository = IndexerRepository::new(&state.pool);
    let indexer_model = repository.get_by_table_name(table_name).await.map_err(IndexerError::InfraError)?;

    let server_port = indexer_model.status_server_port.ok_or(IndexerError::IndexerStatusServerPortNotFound)?;

    // Create a gRPC client
    let endpoint = format!("http://localhost:{}", server_port);
    let mut client = StatusClient::connect(endpoint).await.map_err(IndexerError::FailedToConnectGRPC)?;

    // Create a GetStatusRequest
    let request = tonic::Request::new(GetStatusRequest {});

    // Fetch the status
    let response = client.get_status(request).await.map_err(IndexerError::GRPCRequestFailed)?;

    // Process the response
    let status_response = response.into_inner();

    Ok(Json(status_response.into()))
}
