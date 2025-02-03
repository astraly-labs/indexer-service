pub mod postgres;
pub mod webhook;

use std::process::Stdio;

use axum::async_trait;
use shutil::pipe;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::domain::models::indexer::IndexerError::FailedToStopIndexer;
use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerType};
use crate::handlers::indexers::utils::get_script_tmp_directory;
use crate::utils::env::get_environment_variable;

pub const DEFAULT_STARTING_BLOCK: i64 = 1;

#[async_trait]
pub trait Indexer {
    async fn start(&self, indexer: &IndexerModel) -> Result<u32, IndexerError>;

    #[allow(clippy::result_large_err)]
    fn start_common(&self, binary: String, indexer: &IndexerModel, extra_args: &[&str]) -> Result<u32, IndexerError> {
        let script_path = get_script_tmp_directory(indexer.id);
        let auth_token = get_environment_variable("APIBARA_AUTH_TOKEN");
        let redis_url = get_environment_variable("APIBARA_REDIS_URL");

        let sink_id = indexer.indexer_id.clone().unwrap_or_else(|| indexer.id.to_string());
        let status_server_address = format!("0.0.0.0:{port}", port = indexer.status_server_port.unwrap_or(1234));

        let mut args = vec![
            "run",
            script_path.as_str(),
            "--auth-token",
            auth_token.as_str(),
            "--persist-to-redis",
            redis_url.as_str(),
            "--sink-id",
            sink_id.as_str(),
            "--status-server-address",
            status_server_address.as_str(),
            "--allow-env-from-env",
            "STARTING_BLOCK",
        ];
        args.extend_from_slice(extra_args);

        let mut child_handle = Command::new(binary)
            // Silence  stdout and stderr
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("STARTING_BLOCK", indexer.starting_block.unwrap_or(DEFAULT_STARTING_BLOCK).to_string())
            .args(args)
            .spawn()
            .map_err(|e| IndexerError::FailedToStartIndexer(e.to_string(), indexer.id.to_string()))?;

        let id = child_handle.id().expect("Failed to get the child process id");

        let stdout = child_handle.stdout.take().expect("child did not have a handle to stdout");
        let stderr = child_handle.stderr.take().expect("child did not have a handle to stderr");

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let indexer_id = indexer.id;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = stdout_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => tracing::info!("[indexer-{}-stdout] {}", indexer_id, line),
                            Err(_) => (), // we will break on .wait
                            _ => ()
                        }
                    }
                    result = stderr_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => tracing::info!("[indexer-{}-stderr] {}", indexer_id, line),
                            Err(_) => (), // we will break on .wait
                            _ => ()
                        }
                    }
                    result = child_handle.wait() => {
                        match result.unwrap().success() {
                            true => {
                                tracing::info!("Child process exited successfully {}", indexer_id);
                                // TODO: stop indexer
                            },
                            false => {
                                tracing::error!("Child process exited with an error {}", indexer_id);
                                // let _ = start_indexer(indexer_id, 1).await;
                            }
                        }
                        break // child process exited
                    }
                };
            }
        });

        Ok(id)
    }

    #[allow(clippy::result_large_err)]
    async fn stop(&self, indexer: IndexerModel) -> Result<(), IndexerError> {
        let process_id = match indexer.process_id {
            Some(process_id) => process_id,
            None => {
                return Err(IndexerError::InternalServerError("Cannot stop indexer without process id".to_string()));
            }
        };

        if !self.is_running(indexer.clone()).await? {
            return Err(IndexerError::InternalServerError(format!(
                "Cannot stop indexer that's not running, indexer id {}",
                indexer.id
            )));
        }

        let is_success = Command::new("kill")
            // Silence  stdout and stderr
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args([
                process_id.to_string().as_str(),
            ])
            .spawn()
            .map_err(|_| IndexerError::FailedToStopIndexer(process_id))?
            .wait()
            .await
            .unwrap()
            .success();

        if !is_success {
            return Err(FailedToStopIndexer(process_id));
        }
        Ok(())
    }
    async fn is_running(&self, indexer: IndexerModel) -> Result<bool, IndexerError> {
        let process_id = match indexer.process_id {
            Some(process_id) => process_id,
            None => {
                return Err(IndexerError::InternalServerError(
                    "Cannot check running status for indexer without process id".to_string(),
                ));
            }
        };

        // Check if the process is running and not in the defunct state
        // `Z` state implies the zombie state where the process is technically
        // dead but still in the process table
        Ok(pipe(vec![vec!["ps", "-o", "stat=", "-p", process_id.to_string().as_str()], vec!["grep", "-vq", "Z"]])
            .is_ok())
    }
}

pub fn get_indexer_handler(indexer_type: &IndexerType) -> Box<dyn Indexer + Sync + Send> {
    match indexer_type {
        IndexerType::Webhook => Box::new(webhook::WebhookIndexer {}),
        IndexerType::Postgres => Box::new(postgres::PostgresIndexer {}),
    }
}
