use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AxumErrorResponse {
    pub happened_at: DateTime<Utc>,
    pub message: String,
    pub resource: String,
}
