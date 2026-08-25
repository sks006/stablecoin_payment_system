use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub idempotency_key: String,
    pub status: String,
    pub signature: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MintResponse {
    pub signature: String,
    pub status: String,
}
