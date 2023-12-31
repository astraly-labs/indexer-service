use axum::async_trait;
use sqs_worker::{SQSListener, SQSListenerClientBuilder};

use crate::constants::sqs::{FAILED_INDEXER_QUEUE, START_INDEXER_QUEUE, STOP_INDEXER_QUEUE};
use crate::consumers::{get_credentials, Consumers};
use crate::domain::models::indexer::IndexerError;
use crate::handlers::indexers::fail_indexer::fail_indexer;
use crate::handlers::indexers::start_indexer::start_indexer;
use crate::handlers::indexers::stop_indexer::update_indexer_state;
use crate::types::sqs::{StartIndexerRequest, StopIndexerRequest};

async fn consume_start_indexer() -> Result<(), IndexerError> {
    let (credentials_provider, region) = get_credentials();
    let listener = SQSListener::new(START_INDEXER_QUEUE.into(), |message| {
        tracing::info!("Received message to start indexer: {:?}", message.body());
        let m = message.clone();
        let request: StartIndexerRequest =
            serde_json::from_str(m.body().unwrap()).expect("Invalid message body to start indexer");
        tokio::spawn(async move { start_indexer(request.id, request.attempt_no).await });
    });

    let client = SQSListenerClientBuilder::new_with(region, credentials_provider)
        .listener(listener)
        .build()
        .map_err(IndexerError::FailedToCreateSQSListener)?;
    let _ = client.start().await;

    Ok(())
}

async fn consume_failed_indexer() -> Result<(), IndexerError> {
    let (credentials_provider, region) = get_credentials();
    let listener = SQSListener::new(FAILED_INDEXER_QUEUE.into(), |message| {
        tracing::info!("Received message to set indexer as failed: {:?}", message.body());

        let m = message.clone();
        tokio::spawn(async move {
            fail_indexer(m.body().unwrap().try_into().expect("Invalid message body to fail indexer")).await
        });
    });

    let client = SQSListenerClientBuilder::new_with(region, credentials_provider)
        .listener(listener)
        .build()
        .map_err(IndexerError::FailedToCreateSQSListener)?;
    let _ = client.start().await;

    Ok(())
}

async fn consume_stop_indexer() -> Result<(), IndexerError> {
    let (credentials_provider, region) = get_credentials();
    let listener = SQSListener::new(STOP_INDEXER_QUEUE.into(), |message| {
        tracing::info!("Received message to stop indexer: {:?}", message.body());
        let m = message.clone();
        let request: StopIndexerRequest =
            serde_json::from_str(m.body().unwrap()).expect("Invalid message body to start indexer");
        tokio::spawn(async move { update_indexer_state(request.id, request.status).await });
    });

    let client = SQSListenerClientBuilder::new_with(region, credentials_provider)
        .listener(listener)
        .build()
        .map_err(IndexerError::FailedToCreateSQSListener)?;
    let _ = client.start().await;

    Ok(())
}

pub struct IndexerConsumers;
#[async_trait]
impl Consumers for IndexerConsumers {
    async fn init_consumers() -> Result<(), IndexerError> {
        tokio::spawn(consume_start_indexer());
        tokio::spawn(consume_failed_indexer());
        tokio::spawn(consume_stop_indexer());
        Ok(())
    }
}
