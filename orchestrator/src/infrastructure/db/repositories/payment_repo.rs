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

    pub async fn get_submitted_payments(&self) -> Result<Vec<Payment>, crate::domain::error::Error> {
        let rows = sqlx::query(
            "SELECT id, idempotency_key, amount, sender, recipient, status, signature, created_at, updated_at \
             FROM payments WHERE status = $1"
        )
        .bind(crate::domain::payment::PaymentStatus::Submitted)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;

        let mut payments = Vec::new();
        for row in rows {
            use sqlx::Row;
            let id: uuid::Uuid = row.try_get("id").map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;
            let idempotency_key: String = row.try_get("idempotency_key").map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;
            let amount_i64: i64 = row.try_get("amount").map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;
            let sender: String = row.try_get("sender").map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;
            let recipient: String = row.try_get("recipient").map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;
            let status: crate::domain::payment::PaymentStatus = row.try_get("status").map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;
            let signature: Option<String> = row.try_get("signature").map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;
            let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at").map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;
            let updated_at: chrono::DateTime<chrono::Utc> = row.try_get("updated_at").map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;

            payments.push(Payment {
                id,
                idempotency_key,
                amount: amount_i64 as u64,
                sender,
                recipient,
                status,
                signature,
                created_at,
                updated_at,
            });
        }

        Ok(payments)
    }


    pub async fn update_status(&self, id: uuid::Uuid, status: crate::domain::payment::PaymentStatus) -> Result<(), crate::domain::error::Error> {
        sqlx::query(
            "UPDATE payments SET status = $2, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .bind(status)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;

        Ok(())
    }
}
