pub mod postgres;
pub mod webhook;

use std::process::Stdio;

use axum::async_trait;
use chrono::Utc;
use shutil::pipe;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

use crate::constants::indexers::{MAX_INDEXER_START_RETRIES, WORKING_INDEXER_THRESHOLD_TIME_MINUTES};
use crate::domain::models::indexer::IndexerError::FailedToStopIndexer;
use crate::domain::models::indexer::{IndexerError, IndexerModel, IndexerStatus, IndexerType};
use crate::handlers::indexers::utils::get_script_tmp_directory;
use crate::publishers::indexers::{publish_failed_indexer, publish_start_indexer, publish_stop_indexer};
use crate::utils::env::get_environment_variable;

#[async_trait]
pub trait Indexer {
    async fn start(&self, indexer: &IndexerModel, attempt: u32) -> Result<u32, IndexerError>;

    #[allow(clippy::result_large_err)]
    fn start_common(
        &self,
        binary: String,
        indexer: &IndexerModel,
        attempt: u32,
        extra_args: &[&str],
    ) -> Result<u32, IndexerError> {
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
        args.extend_from_slice(extra_args);

        let indexer_start_time = Utc::now().time();
        let mut child_handle = Command::new(binary)
            // Silence  stdout and stderr
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(args)
            .spawn()
            .map_err(|_| IndexerError::FailedToStartIndexer(indexer_id.clone()))?;

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
                            Ok(Some(line)) => println!("[indexer-{}-stdout] {}", indexer_id, line),
                            Err(_) => (), // we will break on .wait
                            _ => ()
                        }
                    }
                    result = stderr_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => println!("[indexer-{}-stderr] {}", indexer_id, line),
                            Err(_) => (), // we will break on .wait
                            _ => ()
                        }
                    }
                    result = child_handle.wait() => {
                        let indexer_id = Uuid::parse_str(indexer_id.as_str()).expect("Invalid UUID for indexer");
                        match result.unwrap().success() {
                            true => {
                                tracing::info!("Child process exited successfully {}", indexer_id);
                                publish_stop_indexer(indexer_id, IndexerStatus::Stopped).await.unwrap();
                            },
                            false => {
                                tracing::error!("Child process exited with an error {}", indexer_id);
                                let indexer_end_time = Utc::now().time();
                                let indexer_duration = indexer_end_time - indexer_start_time;
                                if indexer_duration.num_minutes() > WORKING_INDEXER_THRESHOLD_TIME_MINUTES {
                                    // if the indexer ran for more than threshold time, we will try to restart it
                                    // with attempt id 1. we don't want to increment the attempt id as this was
                                    // a successful run and a we want MAX_INDEXER_START_RETRIES to restart the indexer
                                    tracing::error!("Indexer {} ran for more than 5 minutes, trying restart", indexer_id);
                                    publish_start_indexer(indexer_id, 1).await.unwrap();
                                } else if attempt >= MAX_INDEXER_START_RETRIES {
                                    publish_failed_indexer(indexer_id).await.unwrap();
                                } else {
                                    // if the indexer ran for less than threshold time, we will try to restart it
                                    // by incrementing the attempt id. we increment the attempt id as this was
                                    // a unsuccessful run and a we don't want to exceed MAX_INDEXER_START_RETRIES
                                    publish_start_indexer(indexer_id, attempt+1).await.unwrap();
                                }

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
            println!("the indexer isn't running!");
            return Err(IndexerError::InternalServerError(format!(
                "Cannot stop indexer that's not running, indexer id {}",
                indexer.id
            )));
        }

        let is_success = Command::new("kill")
            // Silence  stdout and stderr
            // .stdout(Stdio::null())
            // .stderr(Stdio::null())
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::config::{config, config_force_init};
    use crate::constants::indexers::MAX_INDEXER_START_RETRIES;
    use crate::constants::sqs::{FAILED_INDEXER_QUEUE, START_INDEXER_QUEUE};
    use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};
    use crate::handlers::indexers::indexer_types::get_indexer_handler;
    use crate::tests::common::utils::assert_queue_contains_message_with_indexer_id;
    use crate::types::sqs::StartIndexerRequest;

    #[tokio::test]
    async fn start_indexer_retry() {
        config_force_init().await;
        let config = config().await;
        let indexer = IndexerModel {
            id: uuid::Uuid::new_v4(),
            indexer_type: IndexerType::Webhook,
            process_id: None,
            status: IndexerStatus::Created,
            target_url: Some("https://example.com".to_string()),
            table_name: None,
        };

        // clear the sqs queue
        config.sqs_client().purge_queue().queue_url(START_INDEXER_QUEUE).send().await.unwrap();
        config.sqs_client().purge_queue().queue_url(FAILED_INDEXER_QUEUE).send().await.unwrap();

        let handler = get_indexer_handler(&indexer.indexer_type);

        let mut attempt = 1;

        while attempt <= MAX_INDEXER_START_RETRIES {
            // try to start the indexer, it will fail as there is no script loaded
            assert!(handler.start(&indexer, attempt).await.is_ok());

            // sleep for 1 seconds to let the indexer fail
            tokio::time::sleep(Duration::from_secs(1)).await;

            // check if the message is present on the queue
            if attempt < MAX_INDEXER_START_RETRIES {
                let request = StartIndexerRequest { id: indexer.id, attempt_no: attempt + 1 };
                assert_queue_contains_message_with_indexer_id(
                    START_INDEXER_QUEUE,
                    serde_json::to_string(&request).unwrap(),
                )
                .await;
            } else {
                assert_queue_contains_message_with_indexer_id(FAILED_INDEXER_QUEUE, indexer.id.to_string()).await;
            }

            attempt += 1;
        }
    }
}
