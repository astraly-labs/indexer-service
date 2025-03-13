use axum::async_trait;

use crate::domain::models::indexer::{IndexerError, IndexerModel};
use crate::handlers::indexers::indexer_types::Indexer;
use crate::utils::env::get_environment_variable;

pub struct ConsoleIndexer;

#[async_trait]
impl Indexer for ConsoleIndexer {
    async fn start(&self, indexer: &IndexerModel) -> Result<u32, IndexerError> {
        let binary_file = format!("{}/{}", get_environment_variable("BINARY_BASE_PATH"), "sink-console");
        let id = self.start_common(
            binary_file,
            indexer,
            &[],
        )?;
        Ok(id)
    }
}
