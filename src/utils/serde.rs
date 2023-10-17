use std::fmt::Debug;

use serde::Serialize;

use crate::domain::models::indexer::IndexerError;

#[allow(clippy::result_large_err)]
pub fn serialize_request<T>(request: T) -> Result<String, IndexerError>
where
    T: Serialize + Debug,
{
    serde_json::to_string(&request).map_err(|e| {
        IndexerError::FailedToSerialize(format!("Failed to serialize request: {:?}, error: {}", request, e))
    })
}
