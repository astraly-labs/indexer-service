pub mod constants;

use std::str::FromStr;

use axum::async_trait;
use diesel::{Connection, PgConnection, RunQueryDsl};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl as AsyncRunQueryDsl};
use uuid::Uuid;

use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};
use crate::infra::errors::InfraError;
use crate::infra::repositories::indexer_repository::{
    IndexerFilter, NewIndexerDb, Repository, UpdateIndexerStatusAndProcessIdDb, UpdateIndexerStatusDb,
};
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
        RunQueryDsl::execute(query, &mut conn).unwrap_or_else(|_| panic!("Could not create database {}", db_name));

        let test_db_url = format!("{}/{}", base_url, db_name);
        let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(test_db_url.clone());
        let pool = Pool::builder(manager).build().unwrap();

        // Add uuid-ossp extension to the test database
        let mut conn = pool.get().await.expect("Failed to get connection from pool");
        let query = diesel::sql_query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"");
        AsyncRunQueryDsl::execute(query, &mut conn).await.expect("Failed to create uuid-ossp extension");

        run_migrations(test_db_url).await.expect("Failed to run migrations");

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

        RunQueryDsl::execute(diesel::sql_query(disconnect_users.as_str()), &mut conn).unwrap();

        let query = diesel::sql_query(format!("DROP DATABASE {}", self.db_name).as_str());
        RunQueryDsl::execute(query, &mut conn).unwrap_or_else(|_| panic!("Couldn't drop database {}", self.db_name));
    }
}

/// Mock the database
#[derive(Debug)]
pub struct MockRepository {
    indexers: Vec<IndexerModel>,
}

impl MockRepository {
    pub fn _new() -> Self {
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

#[async_trait]
impl Repository for MockRepository {
    async fn insert(&mut self, new_indexer: NewIndexerDb) -> Result<IndexerModel, InfraError> {
        let indexer: IndexerModel = new_indexer.try_into().map_err(InfraError::ParseError)?;
        // Insert the indexer in the mock database
        self.indexers.push(indexer.clone());
        Ok(indexer)
    }
    async fn get(&self, id: Uuid) -> Result<IndexerModel, InfraError> {
        let indexer = self.indexers.iter().find(|indexer| indexer.id == id);
        if let Some(indexer) = indexer { Ok(indexer.clone()) } else { Err(InfraError::NotFound) }
    }
    async fn get_all(&self, filter: IndexerFilter) -> Result<Vec<IndexerModel>, InfraError> {
        // Get all indexers with status filter if provided
        let indexers = match filter.status {
            Some(status) => self
                .indexers
                .iter()
                .filter(|indexer| indexer.status == IndexerStatus::from_str(&status).unwrap())
                .cloned()
                .collect(),
            None => self.indexers.clone(),
        };
        Ok(indexers)
    }
    async fn update_status(&mut self, indexer: UpdateIndexerStatusDb) -> Result<IndexerModel, InfraError> {
        // Update the indexer in the mock database
        let i = self.indexers.iter_mut().find(|other| indexer.id.eq(&other.id)).unwrap();
        i.status = IndexerStatus::from_str(&indexer.status).map_err(InfraError::ParseError)?;
        Ok(i.clone())
    }
    async fn update_status_and_process_id(
        &mut self,
        indexer: UpdateIndexerStatusAndProcessIdDb,
    ) -> Result<IndexerModel, InfraError> {
        // Update the indexer in the self.indexers vector
        let i = self.indexers.iter_mut().find(|other| indexer.id.eq(&other.id)).unwrap();
        i.status = IndexerStatus::from_str(&indexer.status).map_err(InfraError::ParseError)?;
        i.process_id = Some(indexer.process_id);
        Ok(i.clone())
    }
}
