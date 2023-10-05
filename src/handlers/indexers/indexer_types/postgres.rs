use std::env;
use std::process::Stdio;

use axum::async_trait;
use tokio::process::Command;

use crate::domain::models::indexer::IndexerModel;
use crate::handlers::indexers::indexer_types::Indexer;
use crate::handlers::indexers::utils::get_script_tmp_directory;

pub struct PostgresIndexer;

#[async_trait]
impl Indexer for PostgresIndexer {
    async fn start(&self, indexer: IndexerModel) -> u32 {
        let binary_file =
            format!("{}/{}", env::var("BINARY_BASE_PATH").expect("BINARY_BASE_PATH is not set"), "sink-postgres");
        let script_path = get_script_tmp_directory(indexer.id);
        let auth_token = env::var("APIBARA_AUTH_TOKEN").expect("APIBARA_AUTH_TOKEN is not set");
        let postgres_connection_string =
            env::var("APIBARA_POSTGRES_CONNECTION_STRING").expect("APIBARA_POSTGRES_CONNECTION_STRING is not set");

        let child_handle = Command::new(binary_file)
            // Silence  stdout and stderr
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args([
                "run",
                script_path.as_str(),
                "--connection-string",
                postgres_connection_string.as_str(),
                "--table-name",
                indexer.table_name.clone().expect("`table_name` not set for postgres indexer").as_str(),
                "--auth-token",
                auth_token.as_str(),
            ])
            .spawn()
            .expect(format!("Could not start indexer - {}",indexer.id).as_str());

        let id = child_handle.id().expect("Failed to get the child process id");
        self.stream_logs(child_handle, indexer);
        id
    }
}
