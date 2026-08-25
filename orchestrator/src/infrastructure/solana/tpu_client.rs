pub struct TpuClient;

impl TpuClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_transaction(&self, _tx: &solana_sdk::transaction::Transaction) -> Result<(), crate::domain::error::Error> {
        tracing::info!("Sending transaction via TPU client");
        Ok(())
    }

    pub async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash, crate::domain::error::Error> {
        Ok(solana_sdk::hash::Hash::default())
    }
}
