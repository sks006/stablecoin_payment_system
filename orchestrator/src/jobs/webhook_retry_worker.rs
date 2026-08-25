use std::sync::Arc;
use crate::{
    domain::{
        webhook_event::WebhookStatus,
        error::Error,
    },
    infrastructure::{
        db::repositories::webhook_repo::WebhookRepository,
        webhook::sender::WebhookSender,
    },
};
use tokio::{
    sync::broadcast,
    time::Duration,
};
use chrono::Utc;
use tracing::{info, error, warn};

pub struct WebhookRetryWorker {
    webhook_repo: Arc<WebhookRepository>,
    webhook_sender: Arc<WebhookSender>,
    webhook_secret: String,
    webhook_url: String,
    shutdown_rx: broadcast::Receiver<()>,
    poll_interval: Duration,
}

impl WebhookRetryWorker {
    pub fn new(
        webhook_repo: Arc<WebhookRepository>,
        webhook_sender: Arc<WebhookSender>,
        webhook_secret: String,
        webhook_url: String,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            webhook_repo,
            webhook_sender,
            webhook_secret,
            webhook_url,
            shutdown_rx,
            poll_interval: Duration::from_secs(5),
        }
    }

    pub async fn run(mut self) {
        info!("Starting WebhookRetryWorker...");
        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    info!("WebhookRetryWorker received shutdown signal.");
                    break;
                }
                _ = tokio::time::sleep(self.poll_interval) => {
                    if let Err(e) = self.process_retries().await {
                        error!("Error during webhook retry sweep: {:?}", e);
                    }
                }
            }
        }
    }

    async fn process_retries(&self) -> Result<(), Error> {
        let events = self.webhook_repo.get_pending_or_failed_events().await?;
        for event in events {
            if event.status == WebhookStatus::Delivered || event.status == WebhookStatus::MaxRetriesExceeded {
                continue;
            }

            if let Some(next_retry) = event.next_retry_at {
                if next_retry > Utc::now() {
                    continue;
                }
            }

            let timestamp = Utc::now().timestamp() as u64;
            match self.webhook_sender.send(
                &self.webhook_url,
                &self.webhook_secret,
                &event.id.to_string(),
                timestamp,
                &event.payload,
            ).await {
                Ok(_) => {
                    self.webhook_repo.update_status(event.id, WebhookStatus::Delivered, event.retry_count, None).await?;
                    info!("Webhook event {} successfully delivered on retry", event.id);
                }
                Err(e) => {
                    let new_retry_count = event.retry_count + 1;
                    if new_retry_count >= 5 {
                        self.webhook_repo.update_status(event.id, WebhookStatus::MaxRetriesExceeded, new_retry_count, None).await?;
                        warn!("Webhook event {} reached max retries. Mark as MaxRetriesExceeded. Error: {:?}", event.id, e);
                    } else {
                        // exponential backoff: 10 * 2^count seconds
                        let delay_secs = 10 * (2_i64.pow(new_retry_count as u32));
                        let next_retry = Utc::now() + chrono::Duration::seconds(delay_secs);
                        self.webhook_repo.update_status(event.id, WebhookStatus::Failed, new_retry_count, Some(next_retry)).await?;
                        info!("Webhook event {} failed to deliver (retry {}). Next retry at {}. Error: {:?}", event.id, new_retry_count, next_retry, e);
                    }
                }
            }
        }
        Ok(())
    }
}
