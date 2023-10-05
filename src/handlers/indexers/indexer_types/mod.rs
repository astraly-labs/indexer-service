pub mod postgres;
pub mod webhook;

use std::process::Stdio;

use axum::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::domain::models::indexer::IndexerError::FailedToStopIndexer;
use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerType};
use crate::handlers::indexers::utils::get_script_tmp_directory;

use crate::publishers::indexers::publish_failed_indexer;
use crate::utils::env::get_environment_variable;

#[async_trait]
pub trait Indexer {
    async fn start(&self, indexer: IndexerModel) -> u32;

    fn start_common(&self, binary: String, indexer: IndexerModel, extra_args: Vec<&str>) -> u32 {
        let script_path = get_script_tmp_directory(indexer.id);
        let auth_token = get_environment_variable("APIBARA_AUTH_TOKEN");
        let etcd_url = get_environment_variable("APIBARA_ETCD_URL");

        let indexer_id = indexer.id.to_string();
        let mut args = vec![
            "run",
            script_path.as_str(),
            "--auth-token",
            auth_token.as_str(),
            "--persist-to-etcd",
            etcd_url.as_str(),
            "--sink-id",
            indexer_id.as_str(),
        ];
        args.extend_from_slice(extra_args.as_slice());

        let mut child_handle = Command::new(binary)
            // Silence  stdout and stderr
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(args)
            .spawn()
            .unwrap_or_else(|_| panic!("Could not start indexer - {}",indexer.id));

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

    #[allow(clippy::result_large_err)]
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
            .expect(format!("Could not stop indexer - {}",indexer.id).as_str())
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

pub fn get_indexer_handler(indexer_type: &IndexerType) -> Box<dyn Indexer + Sync + Send> {
    match indexer_type {
        IndexerType::Webhook => Box::new(webhook::WebhookIndexer {}),
        IndexerType::Postgres => Box::new(postgres::PostgresIndexer {}),
    }
}
