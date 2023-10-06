use aws_sdk_sqs::Error;
use uuid::Uuid;

use crate::constants::sqs::{FAILED_INDEXER_QUEUE, START_INDEXER_QUEUE};
use crate::domain::models::indexer::IndexerError;
use crate::publishers::send_sqs_message;
use crate::types::sqs::StartIndexerRequest;

pub async fn publish_start_indexer(indexer_id: Uuid, attempt: u32) -> Result<(), IndexerError> {
    tracing::info!("Sending message to start indexer with id: {}, attempt: {}", indexer_id.to_string(), attempt);
    let request = StartIndexerRequest { id: indexer_id, attempt_no: attempt };
    send_sqs_message(
        START_INDEXER_QUEUE,
        serde_json::to_string(&request)
            .map_err(|e| IndexerError::FailedToSerialize(format!("StartIndexerRequest: {:?}, error: {}", request, e)))?
            .as_str(),
    )
    .await
    .map_err(IndexerError::FailedToPushToQueue)?;
    tracing::info!("Sent message to start indexer with id: {}, attempt: {}", indexer_id.to_string(), attempt);
    Ok(())
}

pub async fn publish_failed_indexer(indexer_id: Uuid) -> Result<(), Error> {
    tracing::info!("Sending message to set indexer as failed with id: {}", indexer_id.to_string());
    send_sqs_message(FAILED_INDEXER_QUEUE, indexer_id.to_string().as_str()).await?;
    tracing::info!("Sent message to set indexer as failed with id: {}", indexer_id.to_string());
    Ok(())
}
