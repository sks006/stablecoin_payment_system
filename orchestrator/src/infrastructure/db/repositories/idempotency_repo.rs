use sqlx::PgPool;

pub struct IdempotencyRepository {
    pool: PgPool,
}

impl IdempotencyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
