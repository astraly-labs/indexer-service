use axum::extract::State;
use uuid::Uuid;

use crate::config::config;
use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::handlers::indexers::indexer_types::get_indexer_handler;
use crate::infra::repositories::indexer_repository::{IndexerRepository, Repository, UpdateIndexerStatusDb};
use crate::utils::PathExtractor;
use crate::AppState;

pub async fn stop_indexer(
    State(state): State<AppState>,
    PathExtractor(id): PathExtractor<Uuid>,
) -> Result<(), IndexerError> {
    let mut repository = IndexerRepository::new(&state.pool);
    let indexer_model = repository.get(id).await.map_err(IndexerError::InfraError)?;
    match indexer_model.status {
        IndexerStatus::Running => (),
        _ => return Err(IndexerError::InvalidIndexerStatus(indexer_model.status)),
    }

    let indexer = get_indexer_handler(&indexer_model.indexer_type);

    // Check if command failed because indexer was already stopped, in that case update status to
    // Stopped
    let new_status = match indexer.stop(indexer_model).await {
        Ok(_) => IndexerStatus::Stopped,
        Err(e) => {
            if let IndexerError::InternalServerError(error) = e {
                if error.contains("Cannot stop indexer that's not running") {
                    IndexerStatus::Stopped
                } else {
                    IndexerStatus::FailedStopping
                }
            } else {
                IndexerStatus::FailedStopping
            }
        }
    };

    repository
        .update_status(UpdateIndexerStatusDb { id, status: new_status.to_string() })
        .await
        .map_err(IndexerError::InfraError)?;

    Ok(())
}

/// Updates the status of an indexer to a new stopped state i.e. Stopped or FailedStopping
/// This function is called when the indexer is already stopped and we want to update the status.
/// It's triggered by the stop indexer queue which is called when indexer stops with a success
/// status It's possible that the status was already updated to Stopped/FailStopping if the user
/// called the /stop API. So we have `check_redundant_update_call` to avoid duplicate updates.
pub async fn update_indexer_state(id: Uuid, new_status: IndexerStatus) -> Result<(), IndexerError> {
    let config = config().await;
    let mut repository = IndexerRepository::new(config.pool());
    let indexer_model = repository.get(id).await.map_err(IndexerError::InfraError)?;

    let check_redundant_update_call = |current_status: &IndexerStatus, new_status: IndexerStatus, id: Uuid| {
        if *current_status == new_status {
            // redundant call
            return Ok(());
        }
        Err(IndexerError::InternalServerError(format!(
            "Cannot move from {} to {} for indexer {}",
            current_status, new_status, id
        )))
    };
    match indexer_model.status {
        IndexerStatus::Running => (),
        IndexerStatus::Stopped => {
            check_redundant_update_call(&indexer_model.status, new_status, id)?;
        }
        IndexerStatus::FailedStopping => {
            check_redundant_update_call(&indexer_model.status, new_status, id)?;
        }
        _ => return Err(IndexerError::InvalidIndexerStatus(indexer_model.status)),
    }

    let indexer = get_indexer_handler(&indexer_model.indexer_type);

    match indexer.is_running(indexer_model).await? {
        false => (),
        true => {
            return Err(IndexerError::InternalServerError(
                "Cannot set indexer to stopped if it's still running".into(),
            ));
        }
    };

    repository
        .update_status(UpdateIndexerStatusDb { id, status: new_status.to_string() })
        .await
        .map_err(IndexerError::InfraError)?;

    Ok(())
}
