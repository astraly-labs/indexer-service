use uuid::Uuid;

use crate::config::config;
use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::infra::repositories::indexer_repository;
use crate::infra::repositories::indexer_repository::UpdateIndexerStatusDb;

pub async fn fail_indexer(id: Uuid) -> Result<(), IndexerError> {
    let config = config().await;
    let indexer_model = indexer_repository::get(config.pool(), id).await.map_err(IndexerError::InfraError)?;
    match indexer_model.status {
        IndexerStatus::Running => (),
        _ => return Err(IndexerError::InvalidIndexerStatus(indexer_model.status)),
    }
    indexer_repository::update_status(
        config.pool(),
        UpdateIndexerStatusDb { id, status: IndexerStatus::FailedRunning.to_string() },
    )
    .await
    .map_err(IndexerError::InfraError)?;

    Ok(())
}
