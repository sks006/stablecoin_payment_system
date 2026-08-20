use shared_memory::error::{OrchestratorError, ErrorConversion};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use spl_token::instruction::transfer;
use crate::state_parser::load_mut_vault_state;

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], payload: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let funder_wallet_info = next_account_info(accounts_iter)?;
    let funder_token_info = next_account_info(accounts_iter)?;
    let vault_info = next_account_info(accounts_iter)?;
    let vault_token_info = next_account_info(accounts_iter)?;
    let collateral_mint_info = next_account_info(accounts_iter)?;
    let token_program_info = next_account_info(accounts_iter)?;

    // 1. Signer checks
    if !funder_wallet_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 2. Hardware validation (zero-copy lock)
    let vault_state = load_mut_vault_state(vault_info.clone(), program_id)?;

    // 3. Collateral mint binding (prevents fake-token attacks)
    if collateral_mint_info.key.to_bytes() != vault_state.collateral_mint {
        return Err(ProgramError::from(
            OrchestratorError::CollateralMintMismatch.into_program_error(),
        ));
    }

    // 4. Bounds check (prevents panic on short payload)
    if payload.len() < 16 {
        return Err(ProgramError::from(
            OrchestratorError::InvalidInstructionData.into_program_error(),
        ));
    }

    // 5. Safe deserialization (error wraps into ProgramError)
    let (debt_bytes, collateral_bytes) = payload.split_at(8);
    let debt_amount = u64::from_le_bytes(
        debt_bytes
            .try_into()
            .map_err(|_| ProgramError::from(OrchestratorError::InvalidInstructionData.into_program_error()))?,
    );

    let requested_collateral_deposit = u64::from_le_bytes(
        collateral_bytes
            .try_into()
            .map_err(|_| ProgramError::from(OrchestratorError::InvalidInstructionData.into_program_error()))?,
    );

    // 6. Mathematical risk boundary (fixed-point)------
    let pre_collateral: i128 = vault_state.total_collateral.into();
    let pre_debt: i128 = vault_state.total_debt_shares.into();

    let post_collateral = pre_collateral
        .checked_add(requested_collateral_deposit as i128)
        .ok_or_else(|| ProgramError::from(OrchestratorError::MathOverflow.into_program_error()))?;

    let post_debt = pre_debt
        .checked_add(debt_amount as i128)
        .ok_or_else(|| ProgramError::from(OrchestratorError::MathOverflow.into_program_error()))?;

    if post_debt == 0 {
        return Err(ProgramError::from(OrchestratorError::MathOverflow.into_program_error()));
    }

    let collateral_scaler = post_collateral
        .checked_shl(48)
        .ok_or_else(|| ProgramError::from(OrchestratorError::MathOverflow.into_program_error()))?;

    let new_ratio_bits = collateral_scaler
        .checked_div(post_debt)
        .ok_or_else(|| ProgramError::from(OrchestratorError::MathOverflow.into_program_error()))?;

    let min_ratio_bits = i128::from_le_bytes(vault_state.min_collateral_ratio.bits);
    if new_ratio_bits < min_ratio_bits {
        return Err(ProgramError::from(
            OrchestratorError::CollateralRatioTooLow.into_program_error(),
        ));
    }

    // 7. Physical token transfer (CPI actual token movement)
    let transfer_ix = transfer(
        token_program_info.key,
        funder_token_info.key, // source ATA
        vault_token_info.key,  // destination ATA
        funder_wallet_info.key, // owner
        &[],                   // extra signers (none, owner is transaction signer)
        requested_collateral_deposit,
    )?;

    invoke(
        &transfer_ix,
        &[
            funder_token_info.clone(),
            vault_token_info.clone(),
            funder_wallet_info.clone(),
            token_program_info.clone(),
        ],
    )?;

    // ---- 8. State mutation (only after tokens are moved) ----
    vault_state.total_collateral = post_collateral as u64;
    vault_state.total_debt_shares = post_debt as u64;

    Ok(())
}