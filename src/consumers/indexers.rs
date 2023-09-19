use sqs_worker::{SQSListener, SQSListenerClientBuilder};

use crate::constants::sqs::START_INDEXER_QUEUE;
use crate::consumers::{get_credentials, Consumers};
use crate::handlers::indexers::start_indexer::start_indexer;

async fn consume_start_indexer() {
    let (credentials_provider, region) = get_credentials();
    let listener = SQSListener::new(START_INDEXER_QUEUE.into(), |message| {
        tracing::info!("Received message to start indexer: {:?}", message.body());

        // TODO: can we await here? I am getting async closure is unstable error
        tokio::spawn(start_indexer(message.body().unwrap().try_into().expect("Invalid message body to start indexer")));
    });

    let client = SQSListenerClientBuilder::new_with(region, credentials_provider).listener(listener).build().unwrap();
    let _ = client.start().await;
}

pub struct IndexerConsumers;
impl Consumers for IndexerConsumers {
    fn init_consumers() {
        tokio::spawn(consume_start_indexer());
    }
}
