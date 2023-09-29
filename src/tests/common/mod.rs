use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};

// Keep the database info in mind to drop them later
struct TestContext {
    base_url: String,
    db_name: String,
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
