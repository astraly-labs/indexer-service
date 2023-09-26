pub mod webhook;

use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerType};

pub trait Indexer {
    fn start(&self, indexer: IndexerModel) -> u32;
    #[allow(clippy::result_large_err)]
    fn stop(&self, indexer: IndexerModel) -> Result<(), IndexerError>;
    fn is_running(&self, indexer: IndexerModel) -> bool;
}

pub fn get_indexer_handler(indexer_type: &IndexerType) -> impl Indexer {
    match indexer_type {
        IndexerType::Webhook => webhook::WebhookIndexer,
    }
}
