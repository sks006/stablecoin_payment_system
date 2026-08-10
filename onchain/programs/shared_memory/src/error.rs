// shared_memory/src/error.rs

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestratorError {
    // === MEMORY & ALIGNMENT ===
    InvalidDiscriminator = 6000,
    UnalignedMemoryAccess = 6001,

    // === VALIDATION ===
    OracleStale = 6002,
    OracleConfidenceBreach = 6003,
    CollateralRatioTooLow = 6004,
    
    // === MATH ===
    MathOverflow = 6005,

    // === PDA ===
    InvalidPDA = 6006,
    InvalidPDASigner = 6007,

    
    //=== Accounts ===
    InvalidAccountOwner = 6008,
    InvalidAccountSize = 6009,
    AccountNotMutable = 6010,
    InvalidAccountDiscriminator = 6011,
}

// Abstract trait for cross-program error mapping
pub trait ErrorConversion {
    fn into_program_error(self) -> u64;
}

impl ErrorConversion for OrchestratorError {
    fn into_program_error(self) -> u64 {
        self as u64
    }
}

#[cfg(feature = "anchor-bridge")]
impl From<OrchestratorError> for anchor_lang::prelude::ProgramError {
    fn from(error: OrchestratorError) -> Self {
        anchor_lang::prelude::ProgramError::Custom(error as u32)
    }
}

#[cfg(feature = "anchor-bridge")]
impl From<OrchestratorError> for anchor_lang::error::Error {
    fn from(error: OrchestratorError) -> Self {
        anchor_lang::error::Error::from(anchor_lang::prelude::ProgramError::Custom(error as u32))
    }
}