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

    let indexer = get_indexer_handler(&indexer_model.indexer_type);

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
