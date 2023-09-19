use crate::consumers::indexers::IndexerConsumers;
use sqs_worker::EnvironmentVariableCredentialsProvider;
use std::env;

pub mod indexers;

pub trait Consumers {
    fn init_consumers();
}

fn get_credentials() -> (EnvironmentVariableCredentialsProvider, Option<String>) {
    let region = env::var("AWS_REGION").ok();
    (EnvironmentVariableCredentialsProvider::new(), region)
}

pub fn init_consumers() {
    IndexerConsumers::init_consumers();
}
