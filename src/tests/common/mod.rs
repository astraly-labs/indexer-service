use std::error::Error;

use diesel::pg::Pg;
use diesel::{Connection, PgConnection, RunQueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;

use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};
use crate::run_migrations;

// Keep the database info in mind to drop them later
pub struct TestContext {
    pub base_url: String,
    pub db_name: String,
    pub pool: Pool<AsyncPgConnection>,
}

impl TestContext {
    pub async fn new(base_url: &str, db_name: &str) -> Self {
        // First, connect to postgres db to be able to create our test
        // database.
        let postgres_url = format!("{}/postgres", base_url);
        let mut conn = PgConnection::establish(&postgres_url).expect("Cannot connect to postgres database.");

        // Create a new database for the test
        let query = diesel::sql_query(format!("CREATE DATABASE {}", db_name).as_str());
        query.execute(&mut conn).expect(format!("Could not create database {}", db_name).as_str());

        let test_db_url = format!("{}/{}", base_url, db_name);
        let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(test_db_url.clone());
        let pool = Pool::builder(manager).build().unwrap();

        run_migrations(test_db_url).await;

        Self { base_url: base_url.to_string(), db_name: db_name.to_string(), pool }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let postgres_url = format!("{}/postgres", self.base_url);
        let mut conn = PgConnection::establish(&postgres_url).expect("Cannot connect to postgres database.");

        let disconnect_users = format!(
            "SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = '{}';",
            self.db_name
        );

        diesel::sql_query(disconnect_users.as_str()).execute(&mut conn).unwrap();

        let query = diesel::sql_query(format!("DROP DATABASE {}", self.db_name).as_str());
        query.execute(&mut conn).expect(&format!("Couldn't drop database {}", self.db_name));
    }
}

/// Mock the database
#[derive(Debug)]
pub struct MockRepository {
    indexers: Vec<IndexerModel>,
}

impl MockRepository {
    pub fn new() -> Self {
        Self {
            indexers: vec![
                IndexerModel {
                    id: uuid::Uuid::new_v4(),
                    status: IndexerStatus::Created,
                    indexer_type: IndexerType::Webhook,
                    process_id: None,
                    target_url: "https://example.com".to_string(),
                },
                IndexerModel {
                    id: uuid::Uuid::new_v4(),
                    status: IndexerStatus::Running,
                    indexer_type: IndexerType::Webhook,
                    process_id: Some(123),
                    target_url: "https://example.com".to_string(),
                },
                IndexerModel {
                    id: uuid::Uuid::new_v4(),
                    status: IndexerStatus::FailedRunning,
                    indexer_type: IndexerType::Webhook,
                    process_id: None,
                    target_url: "https://example.com".to_string(),
                },
                IndexerModel {
                    id: uuid::Uuid::new_v4(),
                    status: IndexerStatus::Stopped,
                    indexer_type: IndexerType::Webhook,
                    process_id: None,
                    target_url: "https://example.com".to_string(),
                },
                IndexerModel {
                    id: uuid::Uuid::new_v4(),
                    status: IndexerStatus::FailedStopping,
                    indexer_type: IndexerType::Webhook,
                    process_id: Some(123),
                    target_url: "https://example.com".to_string(),
                },
            ],
        }
    }
}
