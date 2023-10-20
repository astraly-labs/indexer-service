use aws_sdk_sqs::Error;
use uuid::Uuid;

use crate::constants::sqs::{FAILED_INDEXER_QUEUE, START_INDEXER_QUEUE, STOP_INDEXER_QUEUE};
use crate::domain::models::indexer::{IndexerError, IndexerStatus};
use crate::publishers::{send_sqs_message, send_sqs_message_with_delay};
use crate::types::sqs::{StartIndexerRequest, StopIndexerRequest};
use crate::utils::serde::serialize_request;

pub async fn publish_start_indexer(indexer_id: Uuid, attempt: u32, delay_seconds: u16) -> Result<(), IndexerError> {
    tracing::info!(
        "Sending message to start indexer with id: {}, attempt: {}, delay_seconds: {}",
        indexer_id.to_string(),
        attempt,
        delay_seconds
    );
    let request = StartIndexerRequest { id: indexer_id, attempt_no: attempt };
    send_sqs_message_with_delay(START_INDEXER_QUEUE, serialize_request(request)?.as_str(), delay_seconds)
        .await
        .map_err(IndexerError::FailedToPushToQueue)?;
    tracing::info!(
        "Sent message to start indexer with id: {}, attempt: {}, delay_seconds: {}",
        indexer_id.to_string(),
        attempt,
        delay_seconds
    );
    Ok(())
}

pub async fn publish_failed_indexer(indexer_id: Uuid) -> Result<(), Error> {
    tracing::info!("Sending message to set indexer as failed with id: {}", indexer_id.to_string());
    send_sqs_message(FAILED_INDEXER_QUEUE, indexer_id.to_string().as_str()).await?;
    tracing::info!("Sent message to set indexer as failed with id: {}", indexer_id.to_string());
    Ok(())
}

pub async fn publish_stop_indexer(indexer_id: Uuid, status: IndexerStatus) -> Result<(), IndexerError> {
    tracing::info!("Sending message to stop indexer with status: {}, attempt: {}", indexer_id.to_string(), status);
    let request = StopIndexerRequest { id: indexer_id, status };
    send_sqs_message(STOP_INDEXER_QUEUE, serialize_request(request)?.as_str())
        .await
        .map_err(IndexerError::FailedToPushToQueue)?;
    tracing::info!("Sent message to stop indexer with id: {}, status: {}", indexer_id.to_string(), status);
    Ok(())
}
