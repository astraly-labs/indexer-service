use axum::async_trait;

use crate::domain::models::indexer::{IndexerError, IndexerModel};
use crate::handlers::indexers::indexer_types::Indexer;
use crate::utils::env::get_environment_variable;

pub struct PostgresIndexer;

#[async_trait]
impl Indexer for PostgresIndexer {
    async fn start(&self, indexer: &IndexerModel, attempt: u32) -> Result<u32, IndexerError> {
        let binary_file = format!("{}/{}", get_environment_variable("BINARY_BASE_PATH"), "sink-postgres");
        let postgres_connection_string = indexer
            .custom_connection_string
            .clone()
            .unwrap_or_else(|| get_environment_variable("APIBARA_POSTGRES_CONNECTION_STRING"));
        let table_name = indexer.table_name.as_ref().expect("`table_name` not set for postgres indexer");
        let id = self.start_common(
            binary_file,
            indexer,
            attempt,
            &["--connection-string", postgres_connection_string.as_str(), "--table-name", table_name.as_str()],
        )?;
        Ok(id)
    }
}
