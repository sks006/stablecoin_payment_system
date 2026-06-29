use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdempotencyRecord {
    pub key: String,
    pub response_body: serde_json::Value,
}
