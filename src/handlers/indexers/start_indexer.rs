use std::fs;
use std::io::Write;

use axum::extract::State;
use uuid::Uuid;

use crate::config::config;
use crate::constants::s3::INDEXER_SERVICE_BUCKET;
use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::handlers::indexers::indexer_types::get_indexer_handler;
use crate::handlers::indexers::utils::{get_s3_script_key, get_script_tmp_directory};
use crate::infra::repositories::indexer_repository::{
    IndexerFilter, IndexerRepository, Repository, UpdateIndexerStatusAndProcessIdDb,
};
use crate::publishers::indexers::publish_start_indexer;
use crate::utils::PathExtractor;
use crate::AppState;

pub async fn start_indexer(id: Uuid) -> Result<(), IndexerError> {
    let config = config().await;
    let mut repository = IndexerRepository::new(config.pool());
    let indexer_model = repository.get(id).await.map_err(IndexerError::InfraError)?;
    let indexer = get_indexer_handler(&indexer_model.indexer_type);

    match indexer_model.status {
        IndexerStatus::Created => (),
        IndexerStatus::Stopped => (),
        IndexerStatus::FailedRunning => (),
        IndexerStatus::Running => {
            // it's possible that the indexer is in the running state but the process isn't running
            // this can happen when the service restarts in an new machine but the process was still
            // marked as running on the DB
            if indexer.is_running(indexer_model.clone()).await? {
                tracing::info!("Indexer is already running, id {}", indexer_model.id);
                return Ok(());
            }
        }
        _ => return Err(IndexerError::InvalidIndexerStatus(indexer_model.status)),
    }

    let data = config
        .s3_client()
        .get_object()
        .bucket(INDEXER_SERVICE_BUCKET)
        .key(get_s3_script_key(id))
        .send()
        .await
        .map_err(IndexerError::FailedToGetFromS3)?;

    let aggregated_bytes = data.body.collect().await.map_err(IndexerError::FailedToCollectBytesFromS3)?;

    let mut file = fs::File::create(get_script_tmp_directory(id)).map_err(IndexerError::FailedToCreateFile)?;
    file.write_all(aggregated_bytes.into_bytes().to_vec().as_slice()).map_err(IndexerError::FailedToCreateFile)?;

    let process_id = indexer.start(&indexer_model).await.into();

    repository
        .update_status_and_process_id(UpdateIndexerStatusAndProcessIdDb {
            id: indexer_model.id,
            process_id,
            status: IndexerStatus::Running.to_string(),
        })
        .await
        .map_err(IndexerError::InfraError)?;

    Ok(())
}

pub async fn start_indexer_api(
    State(_state): State<AppState>,
    PathExtractor(id): PathExtractor<Uuid>,
) -> Result<(), IndexerError> {
    start_indexer(id).await
}

pub async fn start_all_indexers() -> Result<(), IndexerError> {
    let config = config().await;
    let repository = IndexerRepository::new(config.pool());
    let indexers = repository
        .get_all(IndexerFilter { status: Some(IndexerStatus::Running.to_string()) })
        .await
        .map_err(IndexerError::InfraError)?;

    for indexer in indexers {
        // we can ideally check if the indexer is already running here but if there are a lot of indexers
        // it would be too many db queries at startup, hence we do that inside the start_indexer function
        // which runs by consuming from the SQS queue
        // TODO: Optimize this in the future (async tokio tasks?)
        publish_start_indexer(indexer.id).await.map_err(IndexerError::FailedToPushToQueue)?;
    }

    Ok(())
}
