use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, FromPrimitive, PartialEq, Eq)]
pub enum StablecoinError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Not rent exempt")]
    NotRentExempt,
    #[error("Already initialized")]
    AlreadyInitialized,
    #[error("Not initialized")]
    NotInitialized,
    #[error("Invalid owner")]
    InvalidOwner,
    #[error("Invalid authority")]
    InvalidAuthority,
    #[error("Math overflow")]
    MathOverflow,
    #[error("Insufficient collateral")]
    InsufficientCollateral,
    #[error("Invalid state flags")]
    InvalidStateFlags,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid version")]
    InvalidVersion,
}

impl From<StablecoinError> for ProgramError {
    fn from(e: StablecoinError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
