use std::fs;
use std::io::Write;

// use aws_sdk_s3::primitives::AggregatedBytes;
use axum::extract::State;
use object_store::path::Path;
use uuid::Uuid;

use crate::config::config;
use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::handlers::indexers::indexer_types::get_indexer_handler;
use crate::handlers::indexers::utils::{get_s3_script_key, get_script_tmp_directory};
use crate::infra::repositories::indexer_repository::{
    IndexerFilter, IndexerRepository, Repository, UpdateIndexerStatusAndProcessIdDb,
};
// use crate::utils::env::get_environment_variable;
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

    // let bucket_name = get_environment_variable("INDEXER_SERVICE_BUCKET");

    // let data = config
    //     .s3_client()
    //     .get_object()
    //     .bucket(bucket_name)
    //     .key(get_s3_script_key(id))
    //     .send()
    //     .await
    //     .map_err(IndexerError::FailedToGetFromS3)?;

    let data = config
        .object_store()
        .get(&Path::from(get_s3_script_key(id)))
        .await
        .map_err(IndexerError::FailedToGetFromStore)?;

    let aggregated_bytes = data.bytes().await.map_err(IndexerError::FailedToCollectBytesFromStore)?;

    let mut file = fs::File::create(get_script_tmp_directory(id)).map_err(IndexerError::FailedToCreateFile)?;
    file.write_all(aggregated_bytes.to_vec().as_slice()).map_err(IndexerError::FailedToCreateFile)?;

    let process_id = indexer.start(&indexer_model).await?.into();

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
        // TODO: update indexer status if start fails and not return
        let _ = start_indexer(indexer.id).await;
    }

    Ok(())
}
