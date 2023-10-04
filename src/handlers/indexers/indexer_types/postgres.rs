use axum::async_trait;

use crate::domain::models::indexer::IndexerModel;
use crate::handlers::indexers::indexer_types::Indexer;

pub struct PostgresIndexer;

#[async_trait]
impl Indexer for PostgresIndexer {
    async fn start(&self, indexer: IndexerModel) -> u32 {
        self.start_with_binary(indexer, "sink-postgres").await
    }
}
