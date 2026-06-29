use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MintRequest {
    pub idempotency_key: String,
    pub amount: u64,
    pub recipient: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub idempotency_key: String,
    pub amount: u64,
    pub sender: String,
    pub recipient: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct BurnRequest {
    pub idempotency_key: String,
    pub amount: u64,
    pub owner: String,
}
