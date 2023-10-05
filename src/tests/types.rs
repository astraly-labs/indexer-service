use serde::Deserialize;

#[derive(Deserialize)]
pub struct AxumErrorResponse {
    pub happened_at: String,
    pub message: String,
    pub resource: String,
}
