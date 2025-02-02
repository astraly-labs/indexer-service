use axum::async_trait;

use crate::domain::models::indexer::{IndexerError, IndexerModel};
use crate::handlers::indexers::indexer_types::Indexer;
use crate::utils::env::get_environment_variable;

pub struct WebhookIndexer;

#[async_trait]
impl Indexer for WebhookIndexer {
    async fn start(&self, indexer: &IndexerModel, _attempt: u32) -> Result<u32, IndexerError> {
        let binary_file = format!("{}/{}", get_environment_variable("BINARY_BASE_PATH"), "sink-webhook");
        let id = self.start_common(
            binary_file,
            indexer,
            &["--target-url", indexer.target_url.clone().expect("`target_url` not set for webhook indexer").as_str()],
        )?;
        Ok(id)
    }
}
