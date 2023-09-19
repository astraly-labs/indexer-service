use axum::body::HttpBody;
use axum::extract::State;
use diesel::update;
use uuid::Uuid;

use crate::config::config;
use crate::domain::models::indexer;
use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::handlers::indexers::indexer_types::{get_indexer, Indexer};
use crate::infra::repositories::indexer_repository;
use crate::infra::repositories::indexer_repository::UpdateIndexerStatusAndProcessIdDb;
use crate::AppState;

pub async fn start_indexer(id: Uuid) -> Result<(), IndexerError> {
    let config = config().await;
    let indexer_model = indexer_repository::get(config.pool(), id).await.map_err(IndexerError::InfraError)?;

    match indexer_model.status {
        IndexerStatus::Created => (),
        // TODO: add app level errors on the topmost level
        _ => panic!("Cannot start indexer in state {}", indexer_model.status),
    }

    let indexer = get_indexer(&indexer_model.indexer_type);
    let process_id = indexer.start(indexer_model.clone()) as i64;

    indexer_repository::update_status_and_process_id(
        config.pool(),
        UpdateIndexerStatusAndProcessIdDb {
            id: indexer_model.id,
            process_id,
            status: IndexerStatus::Running.to_string(),
        },
    )
    .await
    .map_err(IndexerError::InfraError)?;

    Ok(())
}
