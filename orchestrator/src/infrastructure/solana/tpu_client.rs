pub struct TpuClient;

impl TpuClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_transaction(&self, _tx: &solana_sdk::transaction::Transaction) -> Result<(), crate::domain::error::Error> {
        tracing::info!("Sending transaction via TPU client");
        Ok(())
    }
}
