use crate::constants::sqs::START_INDEXER_QUEUE;
use crate::publishers::send_sqs_message;
use aws_sdk_sqs::Error;
use uuid::Uuid;

pub async fn publish_start_indexer(indexer_id: Uuid) -> Result<(), Error> {
    tracing::info!("Sending message to start indexer with id: {}", indexer_id.to_string());
    send_sqs_message(START_INDEXER_QUEUE, indexer_id.to_string().as_str()).await?;
    tracing::info!("Sent message to start indexer with id: {}", indexer_id.to_string());
    Ok(())
}
