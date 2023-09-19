use serde::Deserialize;

pub mod create_indexer;

#[derive(Debug, Deserialize)]
pub struct CreateIndexerRequest {
    url: String
}