pub mod webhook;

use axum::async_trait;

use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerType};

#[async_trait]
pub trait Indexer {
    async fn start(&self, indexer: IndexerModel) -> u32;
    #[allow(clippy::result_large_err)]
    async fn stop(&self, indexer: IndexerModel) -> Result<(), IndexerError>;
    async fn is_running(&self, indexer: IndexerModel) -> bool;
}

pub fn get_indexer_handler(indexer_type: &IndexerType) -> impl Indexer {
    match indexer_type {
        IndexerType::Webhook => webhook::WebhookIndexer,
    }
}
