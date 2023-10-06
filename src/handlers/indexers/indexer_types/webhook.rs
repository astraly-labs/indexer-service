use axum::async_trait;

use crate::domain::models::indexer::IndexerModel;
use crate::handlers::indexers::indexer_types::Indexer;
use crate::utils::env::get_environment_variable;

pub struct WebhookIndexer;

#[async_trait]
impl Indexer for WebhookIndexer {
    async fn start(&self, indexer: &IndexerModel, attempt: u32) -> u32 {
        let binary_file = format!("{}/{}", get_environment_variable("BINARY_BASE_PATH"), "sink-webhook");
        self.start_common(
            binary_file,
            indexer,
            attempt,
            &["--target-url", indexer.target_url.clone().expect("`target_url` not set for webhook indexer").as_str()],
        )
    }
}
