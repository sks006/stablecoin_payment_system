use sqlx::PgPool;
use crate::domain::webhook_event::{WebhookEvent, WebhookStatus};
use crate::domain::error::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct WebhookRepository {
    pool: PgPool,
}

impl WebhookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, event: &WebhookEvent) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO webhook_events (id, payment_id, event_type, payload, status, retry_count, next_retry_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (id) DO UPDATE SET \
                status = EXCLUDED.status, \
                retry_count = EXCLUDED.retry_count, \
                next_retry_at = EXCLUDED.next_retry_at, \
                updated_at = EXCLUDED.updated_at"
        )
        .bind(event.id)
        .bind(event.payment_id)
        .bind(&event.event_type)
        .bind(&event.payload)
        .bind(event.status)
        .bind(event.retry_count)
        .bind(event.next_retry_at)
        .bind(event.created_at)
        .bind(event.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_pending_or_failed_events(&self) -> Result<Vec<WebhookEvent>, Error> {
        let rows = sqlx::query_as::<_, WebhookEvent>(
            "SELECT id, payment_id, event_type, payload, status, retry_count, next_retry_at, created_at, updated_at \
             FROM webhook_events \
             WHERE status = 'pending' OR (status = 'failed' AND (next_retry_at IS NULL OR next_retry_at <= NOW()))"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(rows)
    }

    pub async fn update_status(&self, id: Uuid, status: WebhookStatus, retry_count: i32, next_retry_at: Option<DateTime<Utc>>) -> Result<(), Error> {
        sqlx::query(
            "UPDATE webhook_events SET status = $2, retry_count = $3, next_retry_at = $4, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .bind(status)
        .bind(retry_count)
        .bind(next_retry_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }
}
