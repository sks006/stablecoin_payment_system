use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MintRequest {
    pub idempotency_key: String,
    pub funder_wallet: String,
    pub funder_token: String,
    pub vault: String,
    pub vault_token: String,
    pub collateral_mint: String,
    pub debt_amount: u64,
    pub collateral_deposit: u64,
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
