use sqlx::PgPool;
use crate::domain::payment::Payment;

pub struct PaymentRepository {
    pool: PgPool,
}

impl PaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, payment: &Payment) -> Result<(), crate::domain::error::Error> {
        sqlx::query(
            "INSERT INTO payments (id, idempotency_key, amount, sender, recipient, status, signature, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (idempotency_key) DO UPDATE SET status = EXCLUDED.status, signature = EXCLUDED.signature, updated_at = EXCLUDED.updated_at"
        )
        .bind(payment.id)
        .bind(&payment.idempotency_key)
        .bind(payment.amount as i64)
        .bind(&payment.sender)
        .bind(&payment.recipient)
        .bind(payment.status)
        .bind(&payment.signature)
        .bind(payment.created_at)
        .bind(payment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;

        Ok(())
    }
}
