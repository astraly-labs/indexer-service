use std::str::FromStr;

use axum::async_trait;
use diesel::serialize::IsNull::No;
use uuid::Uuid;

use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};
use crate::infra::errors::InfraError;
use crate::infra::repositories::indexer_repository::{
    IndexerFilter, NewIndexerDb, Repository, UpdateIndexerStatusAndProcessIdDb, UpdateIndexerStatusDb,
};

pub mod constants;
pub mod utils;

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
                    target_url: Some("https://example.com".to_string()),
                    table_name: None,
                },
                IndexerModel {
                    id: uuid::Uuid::new_v4(),
                    status: IndexerStatus::Running,
                    indexer_type: IndexerType::Webhook,
                    process_id: Some(123),
                    target_url: Some("https://example.com".to_string()),
                    table_name: None,
                },
                IndexerModel {
                    id: uuid::Uuid::new_v4(),
                    status: IndexerStatus::FailedRunning,
                    indexer_type: IndexerType::Webhook,
                    process_id: None,
                    target_url: Some("https://example.com".to_string()),
                    table_name: None,
                },
                IndexerModel {
                    id: uuid::Uuid::new_v4(),
                    status: IndexerStatus::Stopped,
                    indexer_type: IndexerType::Webhook,
                    process_id: None,
                    target_url: Some("https://example.com".to_string()),
                    table_name: None,
                },
                IndexerModel {
                    id: uuid::Uuid::new_v4(),
                    status: IndexerStatus::FailedStopping,
                    indexer_type: IndexerType::Webhook,
                    process_id: Some(123),
                    target_url: Some("https://example.com".to_string()),
                    table_name: None,
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
