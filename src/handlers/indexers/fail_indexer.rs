use uuid::Uuid;

use crate::config::config;
use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::infra::repositories::indexer_repository::{IndexerRepository, Repository, UpdateIndexerStatusDb};

pub async fn fail_indexer(id: Uuid) -> Result<(), IndexerError> {
    let config = config().await;
    let mut repository = IndexerRepository::new(config.pool());
    let indexer_model = repository.get(id).await.map_err(IndexerError::InfraError)?;
    match indexer_model.status {
        IndexerStatus::Running => (),
        _ => return Err(IndexerError::InvalidIndexerStatus(indexer_model.status)),
    }
    repository
        .update_status(UpdateIndexerStatusDb { id, status: IndexerStatus::FailedRunning.to_string() })
        .await
        .map_err(IndexerError::InfraError)?;

    Ok(())
}
