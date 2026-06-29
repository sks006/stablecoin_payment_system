pub struct JitoClient {
    endpoint: String,
}

impl JitoClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }

    pub async fn send_bundle(&self, _txs: Vec<solana_sdk::transaction::Transaction>) -> Result<(), crate::domain::error::Error> {
        tracing::info!("Sending bundle to Jito at {}", self.endpoint);
        Ok(())
    }
}
