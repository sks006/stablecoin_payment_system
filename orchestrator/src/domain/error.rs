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

impl From<Error> for shared_memory::OrchestratorError {
    fn from(err: Error) -> Self {
        match err {
            Error::IdempotencyCollision => shared_memory::OrchestratorError::IdempotencyConflict,
            Error::Kms(ref s) if s.to_lowercase().contains("timeout") => {
                shared_memory::OrchestratorError::KmsTimeout
            }
            Error::Kms(_) => shared_memory::OrchestratorError::KmsUnavailable,
            Error::Solana(ref s) if s.to_lowercase().contains("expired") => {
                shared_memory::OrchestratorError::BlockhashExpired
            }
            Error::Solana(_) => shared_memory::OrchestratorError::SolanaRpcError,
            Error::Validation(_) => shared_memory::OrchestratorError::InvalidPubkey,
            _ => shared_memory::OrchestratorError::InvalidAccountData,
        }
    }
}
