use axum::async_trait;
use sqs_worker::EnvironmentVariableCredentialsProvider;

use crate::consumers::indexers::IndexerConsumers;
use crate::domain::models::indexer::IndexerError;
use crate::utils::env::get_environment_variable;

pub mod indexers;

#[async_trait]
pub trait Consumers {
    async fn init_consumers() -> Result<(), IndexerError>;
}

fn get_credentials() -> (EnvironmentVariableCredentialsProvider, Option<String>) {
    let region = get_environment_variable("AWS_REGION");
    (EnvironmentVariableCredentialsProvider::new(), Some(region))
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
