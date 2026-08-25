use std::sync::Arc;
use crate::{
    domain::{
        payment::Payment,
        webhook_event::{WebhookEvent, WebhookStatus},
        error::Error,
    },
    infrastructure::{
        db::repositories::webhook_repo::WebhookRepository,
        webhook::sender::WebhookSender,
    },
};
use uuid::Uuid;
use chrono::Utc;

pub struct WebhookDispatcher {
    webhook_repo: Arc<WebhookRepository>,
    webhook_sender: Arc<WebhookSender>,
    webhook_secret: String,
    webhook_url: String,
}

impl WebhookDispatcher {
    pub fn new(
        webhook_repo: Arc<WebhookRepository>,
        webhook_sender: Arc<WebhookSender>,
        webhook_secret: String,
        webhook_url: String,
    ) -> Self {
        Self {
            webhook_repo,
            webhook_sender,
            webhook_secret,
            webhook_url,
        }
    }

    pub async fn dispatch(&self, payment: &Payment, event_type: &str) -> Result<(), Error> {
        let payload = serde_json::json!({
            "payment_id": payment.id,
            "idempotency_key": payment.idempotency_key,
            "amount": payment.amount,
            "sender": payment.sender,
            "recipient": payment.recipient,
            "status": payment.status,
            "signature": payment.signature,
            "created_at": payment.created_at,
            "updated_at": payment.updated_at,
        });

        let event_id = Uuid::new_v4();
        let event = WebhookEvent {
            id: event_id,
            payment_id: payment.id,
            event_type: event_type.to_string(),
            payload: payload.clone(),
            status: WebhookStatus::Pending,
            retry_count: 0,
            next_retry_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save to database (persistent outbox pattern)
        self.webhook_repo.save(&event).await?;

        // Attempt immediate delivery
        let timestamp = Utc::now().timestamp() as u64;
        match self.webhook_sender.send(
            &self.webhook_url,
            &self.webhook_secret,
            &event_id.to_string(),
            timestamp,
            &payload,
        ).await {
            Ok(_) => {
                // Mark as delivered
                self.webhook_repo.update_status(event_id, WebhookStatus::Delivered, 0, None).await?;
                tracing::info!("Webhook event {} delivered successfully", event_id);
            }
            Err(e) => {
                // Backoff for first retry: 10s
                let next_retry = Utc::now() + chrono::Duration::seconds(10);
                self.webhook_repo.update_status(event_id, WebhookStatus::Failed, 1, Some(next_retry)).await?;
                tracing::warn!("Webhook event {} failed to deliver immediately. Scheduled retry. Error: {:?}", event_id, e);
            }
        }

        Ok(())
    }
}
