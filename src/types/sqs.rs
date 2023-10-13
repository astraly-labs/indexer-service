use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::indexer::IndexerStatus;

#[derive(Serialize, Deserialize, Debug)]
pub struct StartIndexerRequest {
    pub id: Uuid,
    pub attempt_no: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StopIndexerRequest {
    pub id: Uuid,
    pub status: IndexerStatus,
}
