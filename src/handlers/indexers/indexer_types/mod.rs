pub mod webhook;

use crate::domain::models::indexer::{IndexerModel, IndexerType};

pub trait Indexer {
    fn start(&self, indexer: IndexerModel) -> u32;
}

pub fn get_indexer(indexer_type: &IndexerType) -> impl Indexer {
    match indexer_type {
        IndexerType::Webhook => webhook::WebhookIndexer,
        _ => unimplemented!("Indexer type {} is not implemented", indexer_type),
    }
}
