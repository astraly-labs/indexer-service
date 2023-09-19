use crate::domain::models::indexer::IndexerModel;
use crate::handlers::indexers::indexer_types::Indexer;
use std::env;
use std::process::{Command, Stdio};

pub struct WebhookIndexer;

impl Indexer for WebhookIndexer {
    fn start(&self, indexer: IndexerModel) -> u32 {
        let binary_file = format!("{}/bin/sink-webhook", env!("CARGO_MANIFEST_DIR"));
        let script_path = format!("{}/scripts/{}.js", env!("CARGO_MANIFEST_DIR"), indexer.id.to_string());
        let stream_url = env::var("APIBARA_DNA_STREAM_URL").expect("APIBARA_DNA_STREAM_URL is not set");

        let child_handle = Command::new(binary_file)
            // Silence  stdout and stderr
            // .stdout(Stdio::null())
            // .stderr(Stdio::null())
            .args([
                "run",
                &format!("{}",script_path),
                "--target-url",
                &stream_url
            ])
            .spawn()
            .expect("Could not start background madara node");

        child_handle.id()
    }
}
