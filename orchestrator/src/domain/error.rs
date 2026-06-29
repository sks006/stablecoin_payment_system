use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Redis error: {0}")]
    Cache(String),
    #[error("Solana RPC error: {0}")]
    Solana(String),
    #[error("KMS error: {0}")]
    Kms(String),
    #[error("Kafka error: {0}")]
    Queue(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Idempotency key collision")]
    IdempotencyCollision,
    #[error("NotFound: {0}")]
    NotFound(String),
}
