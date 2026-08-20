// Target: native_stablecoin/src/state_parser.rs

use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use shared_memory::{
    state::{JitVaultState, VAULT_STATE_DISCRIMINATOR},
    error::{OrchestratorError, ErrorConversion},
};

/// Safely extracts and validates a VaultState from a raw AccountInfo.
pub fn load_mut_vault_state<'a>(
    account: AccountInfo<'a>,
    program_id: &Pubkey,
) -> Result<&'a mut JitVaultState, ProgramError> {
    
    // === GATE 1: The Ownership Verification ===
    // You must verify that the Sealevel VM recognizes your program as the owner.
    // Abstract execution: Compare account.owner to program_id.
    // Failure state: Return OrchestratorError::InvalidAccountOwner.
    if account.owner != program_id {
        return Err(ProgramError::from(OrchestratorError::InvalidAccountOwner.into_program_error()));
    }

    // === GATE 2: The Hardware Mutability Check ===
    // The client transaction must have explicitly requested write privileges.
    // Abstract execution: Check the account.is_writable boolean flag.
    // Failure state: Return OrchestratorError::AccountNotMutable.

    if !account.is_writable {
        return Err(ProgramError::from(OrchestratorError::AccountNotMutable.into_program_error()));
    }

    // === GATE 3: The Memory Sizing Check ===
    // Prevent out-of-bounds reads or under-sized spoofed accounts.
    // Abstract execution: Compare account.data_len() to std::mem::size_of::<VaultState>().
    // Failure state: Return OrchestratorError::InvalidAccountSize.

    if account.data_len() != std::mem::size_of::<JitVaultState>() {
        return Err(ProgramError::from(OrchestratorError::InvalidAccountSize.into_program_error()));
    }

    // === GATE 4: The Zero-Copy Cast ===
    let mut buffer = account.try_borrow_mut_data()?;
    let _check = bytemuck::try_from_bytes::<JitVaultState>(&buffer)
        .map_err(|_| ProgramError::from(OrchestratorError::UnalignedMemoryAccess.into_program_error()))?;
    
    let vault_state = unsafe {
        &mut *(buffer.as_mut_ptr() as *mut JitVaultState)
    };
    
    // === GATE 5: The Discriminator Verification ===
    if vault_state.discriminator != VAULT_STATE_DISCRIMINATOR {
        return Err(ProgramError::from(OrchestratorError::InvalidAccountDiscriminator.into_program_error()));
    }

    Ok(vault_state)
}