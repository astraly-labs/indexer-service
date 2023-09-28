use std::env;
use std::process::Stdio;

use axum::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::domain::models::indexer::IndexerError::FailedToStopIndexer;
use crate::domain::models::indexer::{IndexerError, IndexerModel};
use crate::handlers::indexers::indexer_types::Indexer;
use crate::handlers::indexers::utils::get_script_tmp_directory;
use crate::publishers::indexers::publish_failed_indexer;

pub struct WebhookIndexer;

#[async_trait]
impl Indexer for WebhookIndexer {
    async fn start(&self, indexer: IndexerModel) -> u32 {
        let binary_file = format!("{}/bin/sink-webhook", env!("CARGO_MANIFEST_DIR"));
        let script_path = get_script_tmp_directory(indexer.id);
        let auth_token = env::var("APIBARA_AUTH_TOKEN").expect("APIBARA_AUTH_TOKEN is not set");

        let mut child_handle = Command::new(binary_file)
            // Silence  stdout and stderr
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args([
                "run",
                script_path.as_str(),
                "--target-url",
                indexer.target_url.as_str(),
                "--auth-token",
                auth_token.as_str(),
            ])
            .spawn()
            .expect("Could not start the webhook indexer");

        let id = child_handle.id().expect("Failed to get the child process id");

        let stdout = child_handle.stdout.take().expect("child did not have a handle to stdout");
        let stderr = child_handle.stderr.take().expect("child did not have a handle to stderr");

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = stdout_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => tracing::info!("[indexer-{}-stdout] {}", indexer.id, line),
                            Err(_) => (), // we will break on .wait
                            _ => ()
                        }
                    }
                    result = stderr_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => tracing::info!("[indexer-{}-stderr] {}", indexer.id, line),
                            Err(_) => (), // we will break on .wait
                            _ => ()
                        }
                    }
                    result = child_handle.wait() => {
                        match result.unwrap().success() {
                            true => {
                                tracing::info!("Child process exited successfully {}", indexer.id);
                            },
                            false => {
                                tracing::error!("Child process exited with an error {}", indexer.id);
                                // TODO: safe to unwrap here?
                                publish_failed_indexer(indexer.id).await.unwrap();
                            }
                        }
                        break // child process exited
                    }
                };
            }
        });

        id
    }

    async fn stop(&self, indexer: IndexerModel) -> Result<(), IndexerError> {
        let process_id = match indexer.process_id {
            Some(process_id) => process_id,
            None => panic!("Cannot stop indexer without process id"),
        };
        let is_success = Command::new("kill")
            // Silence  stdout and stderr
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args([
                process_id.to_string().as_str(),
            ])
            .spawn()
            .expect("Could not stop the webhook indexer")
            .wait()
            .await
            .unwrap()
            .success();

        if !is_success {
            return Err(FailedToStopIndexer(process_id));
        }
        Ok(())
    }

    async fn is_running(&self, indexer: IndexerModel) -> bool {
        let process_id = match indexer.process_id {
            Some(process_id) => process_id,
            None => panic!("Cannot check is running without process id"),
        };
        Command::new("ps")
            // Silence  stdout and stderr
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args([
                "-p",
                process_id.to_string().as_str(),
            ])
            .spawn()
            .expect("Could not check the indexer status")
            .wait()
            .await
            .unwrap()
            .success()
    }
}
