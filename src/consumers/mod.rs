use std::env;

use sqs_worker::EnvironmentVariableCredentialsProvider;

use crate::consumers::indexers::IndexerConsumers;

pub mod indexers;

pub trait Consumers {
    fn init_consumers();
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
pub fn init_consumers() {
    IndexerConsumers::init_consumers();
}
