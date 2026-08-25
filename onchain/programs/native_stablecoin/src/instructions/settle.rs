// native_stablecoin/src/instructions/settle.rs

use shared_memory::error::{OrchestratorError, ErrorConversion};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::burn;
use crate::state_parser::load_mut_vault_state;

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], payload: &[u8]) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    // ============================================================
    // PHASE 1: ACCOUNT EXTRACTION (fixed order)
    // ============================================================
    let user_info          = next_account_info(account_iter)?;
    let user_debt_ata_info = next_account_info(account_iter)?;
    let vault_info         = next_account_info(account_iter)?;
    let debt_mint_info     = next_account_info(account_iter)?;
    let token_program_info = next_account_info(account_iter)?;

    // ব্যবহারকারী অবশ্যই স্বাক্ষরকারী
    if !user_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // ============================================================
    // PHASE 2: HARDWARE LOCK & MINT BINDING
    // ============================================================
    let vault_state = load_mut_vault_state(vault_info.clone(), program_id)?;

    // ভল্টের সংরক্ষিত ডেট মিন্টের সাথে সরবরাহকৃত মিন্ট মেলানো
    if debt_mint_info.key.to_bytes() != vault_state.debt_mint {
        return Err(ProgramError::from(
            OrchestratorError::DebtMintMismatch.into_program_error(),
        ));
    }

    // ভল্টের অথরিটির সাথে ব্যবহারকারী মেলানো (প্রয়োজনীয়)
    if user_info.key.to_bytes() != vault_state.authority {
        return Err(ProgramError::from(
            OrchestratorError::Unauthorized.into_program_error(),
        ));
    }

    // ============================================================
    // PHASE 3: PAYLOAD DESERIALIZATION (no unwrap)
    // ============================================================
    if payload.len() < 8 {
        return Err(ProgramError::from(
            OrchestratorError::InvalidInstructionData.into_program_error(),
        ));
    }
    let owner_shares_to_settle = u64::from_le_bytes(
        payload[0..8].try_into().map_err(|_| {
            ProgramError::from(OrchestratorError::InvalidInstructionData.into_program_error())
        })?,
    ) as i128;

    // ============================================================
    // PHASE 4: READ ACCUMULATED INTEREST INDEX (fixed-point)
    // ============================================================
    // i128::from_le_bytes ব্যবহার করা হয়েছে (কোনো .bits as i128 নয়)
    let acc_interest_bits = i128::from_le_bytes(vault_state.accumulated_interest_index.bits);

    // ============================================================
    // PHASE 5: CALCULATE REQUIRED TOKEN AMOUNT TO BURN
    // ============================================================
    // Required Tokens = (Shares * Accumulated Interest Index) / 2^48
    let required_tokens_fp = owner_shares_to_settle
        .checked_mul(acc_interest_bits)
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;
    let required_tokens = required_tokens_fp
        .checked_shr(48)
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;

    // নিশ্চিত করি ব্যবহারকারী যথেষ্ট টোকেন পোড়াতে পারে
    if required_tokens <= 0 {
        return Err(ProgramError::from(
            OrchestratorError::InvalidInstructionData.into_program_error(),
        ));
    }

    // ============================================================
    // PHASE 6: BURN DEBT TOKENS FROM USER (standard invoke)
    // ============================================================
    // ব্যবহারকারী একজন সাধারণ সাইনার, তাই invoke ব্যবহার
    let burn_ix = burn(
        token_program_info.key,
        user_debt_ata_info.key,
        debt_mint_info.key,
        user_info.key,          // authority
        &[],                    // কোনো অতিরিক্ত সাইনার নেই
        required_tokens as u64,
    )?;

    invoke(
        &burn_ix,
        &[
            user_debt_ata_info.clone(),
            debt_mint_info.clone(),
            user_info.clone(),
            token_program_info.clone(),
        ],
    )?;

    // ============================================================
    // PHASE 7: STATE MUTATION (debt shares reduction)
    // ============================================================
    vault_state.total_debt_shares = vault_state
        .total_debt_shares
        .checked_sub(owner_shares_to_settle as u64)
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;

    Ok(())
}