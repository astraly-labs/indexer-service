use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct StartIndexerRequest {
    pub id: Uuid,
    pub attempt_no: u32,
}
