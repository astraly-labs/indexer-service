use axum::body::HttpBody;
use axum::extract::State;
use diesel::update;
use uuid::Uuid;

use crate::config::config;
use crate::domain::models::indexer;
use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::handlers::indexers::indexer_types::{get_indexer_handler, Indexer};
use crate::infra::repositories::indexer_repository;
use crate::infra::repositories::indexer_repository::{UpdateIndexerStatusAndProcessIdDb, UpdateIndexerStatusDb};
use crate::AppState;

pub async fn fail_indexer(id: Uuid) -> Result<(), IndexerError> {
    let config = config().await;
    let indexer_model = indexer_repository::get(config.pool(), id).await.map_err(IndexerError::InfraError)?;
    match indexer_model.status {
        IndexerStatus::Running => (),
        // TODO: add app level errors on the topmost level
        _ => panic!("Cannot fail running indexer in state {}", indexer_model.status),
    }
    indexer_repository::update_status(
        config.pool(),
        UpdateIndexerStatusDb { id, status: IndexerStatus::FailedRunning.to_string() },
    )
    .await
    .map_err(IndexerError::InfraError)?;

    Ok(())
}
