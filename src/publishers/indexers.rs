use aws_sdk_sqs::Error;
use uuid::Uuid;

use crate::constants::sqs::{FAILED_INDEXER_QUEUE, START_INDEXER_QUEUE, STOP_INDEXER_QUEUE};
use crate::publishers::send_sqs_message;

pub async fn publish_start_indexer(indexer_id: Uuid) -> Result<(), Error> {
    tracing::info!("Sending message to start indexer with id: {}", indexer_id.to_string());
    send_sqs_message(START_INDEXER_QUEUE, indexer_id.to_string().as_str()).await?;
    tracing::info!("Sent message to start indexer with id: {}", indexer_id.to_string());
    Ok(())
}

pub async fn publish_failed_indexer(indexer_id: Uuid) -> Result<(), Error> {
    tracing::info!("Sending message to set indexer as failed with id: {}", indexer_id.to_string());
    send_sqs_message(FAILED_INDEXER_QUEUE, indexer_id.to_string().as_str()).await?;
    tracing::info!("Sent message to set indexer as failed with id: {}", indexer_id.to_string());
    Ok(())
}

pub async fn publish_stop_indexer(indexer_id: Uuid) -> Result<(), Error> {
    tracing::info!("Sending message to set indexer as failed with id: {}", indexer_id.to_string());
    send_sqs_message(STOP_INDEXER_QUEUE, indexer_id.to_string().as_str()).await?;
    tracing::info!("Sent message to set indexer as failed with id: {}", indexer_id.to_string());
    Ok(())
}
