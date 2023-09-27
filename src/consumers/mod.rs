use std::env;

use sqs_worker::EnvironmentVariableCredentialsProvider;

use crate::consumers::indexers::IndexerConsumers;
use crate::domain::models::indexer::IndexerError;

pub mod indexers;

pub trait Consumers {
    async fn init_consumers() -> Result<(), IndexerError>;
}

fn get_credentials() -> (EnvironmentVariableCredentialsProvider, Option<String>) {
    let region = env::var("AWS_REGION").ok();
    (EnvironmentVariableCredentialsProvider::new(), region)
}

/// Initialize SQS consumers
///
/// Initialize 2 SQS consumers:
/// * Start indexer consumer
/// * Failed indexer consumer
pub async fn init_consumers() -> Result<(), IndexerError> {
    IndexerConsumers::init_consumers().await?;
    Ok(())
}
