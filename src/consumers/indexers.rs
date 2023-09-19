use crate::constants::sqs::START_INDEXER_QUEUE;
use aws_sdk_sqs::{Client, Error};

pub async fn consume_start_indexer(client: &Client) -> Result<(), Error> {}
