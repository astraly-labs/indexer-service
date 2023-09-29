use crate::domain::models::indexer::{IndexerStatus, IndexerType};
use crate::infra::repositories::indexer_repository::{
    IndexerFilter, IndexerRepository, NewIndexerDb, Repository, UpdateIndexerStatusDb,
};
use crate::tests::common::constants::{LOCAL_DB_URL, TEST_DB_NAME};
use crate::tests::common::TestContext;

#[tokio::test]
async fn test_get_indexer() {
    let TestContext { ref pool, .. } = TestContext::new(LOCAL_DB_URL, TEST_DB_NAME).await;

    let mut repository = IndexerRepository::new(pool);

    let id = uuid::Uuid::new_v4();

    // Insert in DB
    let _ = repository
        .insert(NewIndexerDb {
            id,
            status: "Created".to_string(),
            indexer_type: "Webhook".to_string(),
            target_url: "https://example.com".to_string(), // TODO: Mock webhook and test its behavior
        })
        .await
        .unwrap();

    // Retrieve in DB
    let inserted = repository.get(id).await.unwrap();

    assert_eq!(inserted.id, id);
    assert_eq!(inserted.status, IndexerStatus::Created);
    assert_eq!(inserted.indexer_type, IndexerType::Webhook);
    assert_eq!(inserted.target_url, "https://example.com".to_string());
    assert_eq!(inserted.process_id, None);
}

#[tokio::test]
async fn test_insert_indexer() {
    let TestContext { ref pool, .. } = TestContext::new(LOCAL_DB_URL, TEST_DB_NAME).await;
    let mut repository = IndexerRepository::new(pool);
    let id = uuid::Uuid::new_v4();

    // Insert in DB
    let inserted = repository
        .insert(NewIndexerDb {
            id,
            status: "Created".to_string(),
            indexer_type: "Webhook".to_string(),
            target_url: "https://example.com".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(inserted.id, id);
    assert_eq!(inserted.status, IndexerStatus::Created);
    assert_eq!(inserted.indexer_type, IndexerType::Webhook);
    assert_eq!(inserted.target_url, "https://example.com".to_string());
    assert_eq!(inserted.process_id, None);
}

#[tokio::test]
async fn test_update_status() {
    let TestContext { ref pool, .. } = TestContext::new(LOCAL_DB_URL, TEST_DB_NAME).await;
    let mut repository = IndexerRepository::new(pool);
    let id = uuid::Uuid::new_v4();

    // Insert in DB
    let _ = repository
        .insert(NewIndexerDb {
            id,
            status: "Created".to_string(),
            indexer_type: "Webhook".to_string(),
            target_url: "https://example.com".to_string(),
        })
        .await
        .unwrap();

    // Update status in DB
    let updated = repository.update_status(UpdateIndexerStatusDb { id, status: "Running".to_string() }).await.unwrap();

    assert_eq!(updated.id, id);
    assert_eq!(updated.status, IndexerStatus::Running);
}

#[tokio::test]
async fn test_get_all_indexers() {
    let TestContext { ref pool, .. } = TestContext::new(LOCAL_DB_URL, TEST_DB_NAME).await;
    let mut repository = IndexerRepository::new(pool);

    // Insert multiple indexers in DB
    for _ in 0..5 {
        let id = uuid::Uuid::new_v4();
        repository
            .insert(NewIndexerDb {
                id,
                status: "Created".to_string(),
                indexer_type: "Webhook".to_string(),
                target_url: "https://example.com".to_string(),
            })
            .await
            .unwrap();
    }

    let id = uuid::Uuid::new_v4();
    repository
        .insert(NewIndexerDb {
            id,
            status: "Running".to_string(),
            indexer_type: "Webhook".to_string(),
            target_url: "https://example.com".to_string(),
        })
        .await
        .unwrap();

    // Retrieve all indexers with "Created"
    let indexers = repository.get_all(IndexerFilter { status: Some("Created".to_string()) }).await.unwrap();

    assert_eq!(indexers.len(), 5);

    // Retrieve all indexers without filter
    let indexers = repository.get_all(IndexerFilter { status: None }).await.unwrap();

    assert_eq!(indexers.len(), 6);

    // Retrieve all indexers with "Running" filter
    let indexers = repository.get_all(IndexerFilter { status: Some("Running".to_string()) }).await.unwrap();

    assert_eq!(indexers.len(), 1);
}
