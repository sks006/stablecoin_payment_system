use sqlx::PgPool;

pub struct IdempotencyRepository {
    pool: PgPool,
}

impl IdempotencyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn acquire_lock(&self, key: &str) -> Result<(), crate::domain::error::Error> {
        let dummy_response = serde_json::json!({ "status": "processing" });
        
        let result = sqlx::query(
            "INSERT INTO idempotency_keys (key, response_body) VALUES ($1, $2)"
        )
        .bind(key)
        .bind(dummy_response)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(db_err) = e.as_database_error() {
                    if db_err.code().as_deref() == Some("23505") {
                        return Err(crate::domain::error::Error::IdempotencyCollision);
                    }
                }
                Err(crate::domain::error::Error::Database(e.to_string()))
            }
        }
    }

    pub async fn update_tx_id(&self, key: &str, tx_id: &str) -> Result<(), crate::domain::error::Error> {
        let response = serde_json::json!({ "status": "submitted", "tx_id": tx_id });

        sqlx::query(
            "UPDATE idempotency_keys SET response_body = $2 WHERE key = $1"
        )
        .bind(key)
        .bind(response)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;

        sqlx::query(
            "UPDATE payments SET signature = $2, status = $3 WHERE idempotency_key = $1"
        )
        .bind(key)
        .bind(tx_id)
        .bind(crate::domain::payment::PaymentStatus::Submitted)
        .execute(&self.pool)
        .await
        .map_err(|e| crate::domain::error::Error::Database(e.to_string()))?;

        Ok(())
    }
}
