use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

pub struct KmsClient {
    pub key_id: String,
    pubkey: Pubkey,
}

impl KmsClient {
    pub fn new(key_id: &str) -> Self {
        Self {
            key_id: key_id.to_string(),
            pubkey: Pubkey::new_unique(),
        }
    }

    pub fn get_pubkey(&self) -> Pubkey {
        self.pubkey
    }

    pub async fn sign(&self, message: &[u8]) -> Result<Vec<u8>, crate::domain::error::Error> {
        // AWS KMS signing logic placeholder
        tracing::info!("Signing message via KMS key {}", self.key_id);
        Ok(message.to_vec())
    }

    pub async fn sign_transaction(&self, message: &solana_sdk::message::Message) -> Result<Signature, crate::domain::error::Error> {
        let message_bytes = message.serialize();
        let sig_bytes = self.sign(&message_bytes).await?;
        let mut signature_bytes = [0u8; 64];
        let len = sig_bytes.len().min(64);
        signature_bytes[..len].copy_from_slice(&sig_bytes[..len]);
        Ok(Signature::from(signature_bytes))
    }
}
