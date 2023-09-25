use axum::body::HttpBody;
use axum::extract::State;
use diesel::update;
use std::fs;
use std::io::Write;
use uuid::Uuid;

use crate::config::config;
use crate::constants::s3::INDEXER_SERVICE_BUCKET;
use crate::domain::models::indexer;
use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::handlers::indexers::indexer_types::{get_indexer_handler, Indexer};
use crate::handlers::indexers::utils::{get_s3_script_key, get_script_tmp_directory};
use crate::infra::repositories::indexer_repository;
use crate::infra::repositories::indexer_repository::{IndexerFilter, UpdateIndexerStatusAndProcessIdDb};
use crate::publishers::indexers::publish_start_indexer;
use crate::AppState;

pub async fn start_indexer(id: Uuid) -> Result<(), IndexerError> {
    let config = config().await;
    let indexer_model = indexer_repository::get(config.pool(), id).await.map_err(IndexerError::InfraError)?;
    let indexer = get_indexer_handler(&indexer_model.indexer_type);

    match indexer_model.status {
        IndexerStatus::Created => (),
        IndexerStatus::Stopped => (),
        IndexerStatus::Running => {
            // it's possible that the indexer is in the running state but the process isn't running
            // this can happen when the service restarts in an new machine but the process was still
            // marked as running on the DB
            if indexer.is_running(indexer_model.clone()) {
                tracing::info!("Indexer is already running, id {}", indexer_model.id);
                return Ok(());
            }
        }
        // TODO: add app level errors on the topmost level
        _ => panic!("Cannot start indexer in state {}", indexer_model.status),
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

pub async fn start_all_indexers() -> Result<(), IndexerError> {
    let config = config().await;
    let indexers =
        indexer_repository::get_all(config.pool(), IndexerFilter { status: Some(IndexerStatus::Running.to_string()) })
            .await
            .map_err(IndexerError::InfraError)?;

    for indexer in indexers {
        // we can ideally check if the indexer is already running here but if there are a lot of indexers
        // it would be too many db queries at startup, hence we do that inside the start_indexer function
        // which runs by consuming from the SQS queue
        publish_start_indexer(indexer.id).await.map_err(IndexerError::FailedToPushToQueue)?;
    }

    Ok(())
}
