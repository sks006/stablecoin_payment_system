// Target: native_stablecoin/src/instructions/mint_jit.rs

use shared_memory::error::{OrchestratorError, ErrorConversion};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use shared_memory::state::FixedPoint;
use crate::state_parser::load_mut_vault_state;

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], payload: &[u8]) -> ProgramResult {
    // === 1. The Iterator Extraction ===
    // Safely extract the Funder, Vault, Collateral Mint, and Token Program accounts.

    let account_iter = &mut accounts.iter();

    let _funder_info = next_account_info(account_iter)?;
    let vault_info = next_account_info(account_iter)?;
    let _collateral_mint_info = next_account_info(account_iter)?;
    let _token_program_info = next_account_info(account_iter)?;

    // === 2. The Hardware Validation ===
    // Execute state_parser::load_mut_vault_state(vault_info, program_id)?
    // This yields the secure RefMut<'_, JitVaultState>.
    let vault_state = load_mut_vault_state(vault_info, program_id)?;

    // === 3. Payload Deserialization ===
    // Extract the requested `debt_amount` and `collateral_deposit` (as u64s)
    // directly from the payload slice using try_into() on raw byte chunks.
    let (debt_amount_bytes, rest) = payload.split_at(8);
    let (collateral_deposit_bytes, _remaining) = rest.split_at(8);

    let requested_debt_amount = u64::from_le_bytes(debt_amount_bytes.try_into().unwrap());
    let requested_collateral_deposit =
        u64::from_le_bytes(collateral_deposit_bytes.try_into().unwrap());
    
    // === STEP 4: The Mathematical Risk Boundary (Fixed‑Point Correct) ===

    // 1. Project post-transaction balances (cast to i128 for precision)
    let pre_collateral: i128 = vault_state.total_collateral.into();
    let pre_debt: i128 = vault_state.total_debt_shares.into();
    let post_collateral = pre_collateral
        .checked_add(requested_collateral_deposit as i128)
        .ok_or_else(|| ProgramError::from(OrchestratorError::MathOverflow.into_program_error()))?;
    let post_debt = pre_debt
        .checked_add(requested_debt_amount as i128)
        .ok_or_else(|| ProgramError::from(OrchestratorError::MathOverflow.into_program_error()))?;

    // Prevent division by zero (debt must always exist)
    if post_debt == 0 {
        return Err(ProgramError::from(OrchestratorError::MathOverflow.into_program_error()));
    }

    // 2. Precision shift: collateral * 2^48 to align with fixed‑point ratio format
    let collateral_scaled = post_collateral
        .checked_shl(48)
        .ok_or_else(|| ProgramError::from(OrchestratorError::MathOverflow.into_program_error()))?;

    // 3. Safe division: scaled collateral / debt yields a fixed‑point ratio (I80F48)
    let new_ratio_bits = collateral_scaled
        .checked_div(post_debt)
        .ok_or_else(|| ProgramError::from(OrchestratorError::MathOverflow.into_program_error()))?;

    // 4. Extract the vault’s **stored** minimum collateral ratio (from memory, not hardcoded)
    let min_ratio_bits = vault_state.min_collateral_ratio.to_bits(); // i128 already in I80F48 format

    // 5. Threshold check: reject if new ratio is below the vault’s risk boundary
    if new_ratio_bits < min_ratio_bits {
        return Err(ProgramError::from(OrchestratorError::CollateralRatioTooLow.into_program_error()));
    }

    // === 6. State Mutation ===
    // vault.total_collateral += collateral_deposit;
    // vault.total_debt_shares += debt_amount;
    vault_state.total_collateral = post_collateral as u64;
    vault_state.total_debt_shares = post_debt as u64;

    Ok(())
}
