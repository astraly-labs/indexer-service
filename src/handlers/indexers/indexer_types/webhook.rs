use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::domain::models::indexer::IndexerError::FailedToStopIndexer;
use crate::domain::models::indexer::{IndexerError, IndexerModel};
use crate::handlers::indexers::indexer_types::Indexer;
use crate::handlers::indexers::utils::get_script_tmp_directory;
use crate::publishers::indexers::publish_failed_indexer;

pub struct WebhookIndexer;

impl Indexer for WebhookIndexer {
    fn start(&self, indexer: IndexerModel) -> u32 {
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

        let id = child_handle.id();

        tokio::spawn(async move {
            // all success messages are logged first, if error messages
            // occur between success messages they won't be logged till
            // the indexer is stopped
            if let Some(ref mut stdout) = child_handle.stdout {
                for line in BufReader::new(stdout).lines() {
                    let line = line.unwrap();
                    println!("[indexer-{}-stdout] {}", indexer.id, line);
                }
            }

            if let Some(ref mut stderr) = child_handle.stderr {
                for line in BufReader::new(stderr).lines() {
                    let line = line.unwrap();
                    println!("[indexer-{}-stderr] {}", indexer.id, line);
                }
            }

            let exit_status = child_handle.wait();
            if exit_status.unwrap().success() {
                tracing::info!("Child process exited successfully {}", indexer.id);
            } else {
                tracing::error!("Child process exited with an error {}", indexer.id);
                // TODO: safe to unwrap here?
                publish_failed_indexer(indexer.id).await.unwrap();
            }
        });

        id
    }

    fn stop(&self, indexer: IndexerModel) -> Result<(), IndexerError> {
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
            .unwrap()
            .success();

        if !is_success {
            return Err(FailedToStopIndexer(process_id));
        }
        Ok(())
    }

    fn is_running(&self, indexer: IndexerModel) -> bool {
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
            .unwrap()
            .success()
    }
}
