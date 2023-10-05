use axum::async_trait;

use crate::domain::models::indexer::IndexerModel;
use crate::handlers::indexers::indexer_types::Indexer;
use crate::utils::env::get_environment_variable;

pub struct PostgresIndexer;

#[async_trait]
impl Indexer for PostgresIndexer {
    async fn start(&self, indexer: &IndexerModel) -> u32 {
        let binary_file = format!("{}/{}", get_environment_variable("BINARY_BASE_PATH"), "sink-postgres");
        let postgres_connection_string = get_environment_variable("APIBARA_POSTGRES_CONNECTION_STRING");
        let table_name = indexer.table_name.clone().expect("`table_name` not set for postgres indexer");
        self.start_common(
            binary_file,
            &indexer,
            &["--connection-string", postgres_connection_string.as_str(), "--table-name", table_name.as_str()],
        )
    }
}
