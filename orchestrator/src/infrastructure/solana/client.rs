use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Signature,
    transaction::Transaction,
};
use std::str::FromStr;

pub struct SolanaClient {
    client: RpcClient,
}

impl SolanaClient {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            client: RpcClient::new_with_commitment(
                rpc_url.to_string(),
                CommitmentConfig::confirmed(),
            ),
        }
    }

    pub async fn send_transaction(&self, tx: &Transaction) -> Result<Signature, crate::domain::error::Error> {
        self.client
            .send_transaction(tx)
            .await
            .map_err(|e| crate::domain::error::Error::Solana(e.to_string()))
    }

    pub async fn confirm_transaction(&self, signature: &str) -> Result<bool, crate::domain::error::Error> {
        let sig = Signature::from_str(signature)
            .map_err(|e| crate::domain::error::Error::Validation(e.to_string()))?;
        
        self.client
            .confirm_transaction(&sig)
            .await
            .map_err(|e| crate::domain::error::Error::Solana(e.to_string()))
    }
}
