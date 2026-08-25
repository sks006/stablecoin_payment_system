use reqwest::Client;
use crate::domain::error::Error;

pub struct WebhookSender {
    client: Client,
}

impl WebhookSender {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn send(
        &self,
        url: &str,
        secret: &str,
        event_id: &str,
        timestamp: u64,
        payload: &serde_json::Value,
    ) -> Result<(), Error> {
        let payload_str = payload.to_string();
        let signature = crate::infrastructure::webhook::signature::calculate_signature(secret, timestamp, &payload_str);

        let response = self.client.post(url)
            .header("X-Webhook-Signature", signature)
            .header("X-Webhook-Timestamp", timestamp.to_string())
            .header("X-Webhook-Event-Id", event_id)
            .json(payload)
            .send()
            .await
            .map_err(|e| Error::Infrastructure(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::Infrastructure(format!("Received status code {}", response.status())))
        }
    }
}
