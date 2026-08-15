// native_stablecoin/src/instructions/liquidate.rs

use shared_memory::error::{OrchestratorError, ErrorConversion};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
};
use spl_token::{
    instruction::{burn, transfer},
    state::Mint,
};
use pyth_sdk_solana::load_price_feed_from_account_info;
use crate::state_parser::load_mut_vault_state;

const MAX_STALENESS_SECONDS: i64 = 60;
const MAX_CONFIDENCE_PERCENT: u64 = 1;

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], payload: &[u8]) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    // ============================================================
    // PHASE 1: ACCOUNT EXTRACTION
    // ============================================================
    let keeper_info               = next_account_info(account_iter)?;
    let keeper_debt_ata_info      = next_account_info(account_iter)?;
    let keeper_collateral_ata_info = next_account_info(account_iter)?;
    let vault_info                = next_account_info(account_iter)?;
    let vault_collateral_ata_info  = next_account_info(account_iter)?;
    let collateral_mint_info      = next_account_info(account_iter)?;
    let debt_mint_info            = next_account_info(account_iter)?;
    let oracle_info               = next_account_info(account_iter)?;
    let token_program_info        = next_account_info(account_iter)?;

    if !keeper_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // ============================================================
    // PHASE 2: HARDWARE LOCK & MINT BINDING
    // ============================================================
    let vault_state = load_mut_vault_state(vault_info, program_id)?;

    // ফিক্স ৩: মিন্ট বাইন্ডিং যাচাই
    if collateral_mint_info.key.to_bytes() != vault_state.collateral_mint {
        return Err(ProgramError::from(
            OrchestratorError::CollateralMintMismatch.into_program_error(),
        ));
    }
    if debt_mint_info.key.to_bytes() != vault_state.debt_mint {
        return Err(ProgramError::from(
            OrchestratorError::DebtMintMismatch.into_program_error(),
        ));
    }

    // ============================================================
    // PHASE 3: ORACLE SANITIZATION
    // ============================================================
    if oracle_info.owner != &pyth_sdk_solana::ID {
        return Err(ProgramError::from(
            OrchestratorError::InvalidOracleOwner.into_program_error(),
        ));
    }

    let price_feed = load_price_feed_from_account_info(oracle_info).map_err(|_| {
        ProgramError::from(OrchestratorError::OraclePriceUnavailable.into_program_error())
    })?;
    let current_price = price_feed.get_current_price().ok_or_else(|| {
        ProgramError::from(OrchestratorError::OraclePriceUnavailable.into_program_error())
    })?;

    let clock = Clock::get()?;
    let staleness = clock
        .unix_timestamp
        .checked_sub(current_price.publish_time)
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::StaleOraclePrice.into_program_error()
        ))?;
    if staleness > MAX_STALENESS_SECONDS {
        return Err(ProgramError::from(
            OrchestratorError::StaleOraclePrice.into_program_error(),
        ));
    }

    let price_abs = current_price.price.unsigned_abs();
    let max_allowed_confidence = price_abs
        .checked_mul(MAX_CONFIDENCE_PERCENT)
        .and_then(|v| v.checked_div(100))
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;
    if current_price.conf > max_allowed_confidence {
        return Err(ProgramError::from(
            OrchestratorError::OracleConfidenceTooWide.into_program_error(),
        ));
    }

    // ফিক্স ২: স্যানিটাইজড ভেরিয়েবল সংরক্ষণ
    let sanitized_price = current_price.price;
    let sanitized_expo = current_price.expo;

    // ============================================================
    // PHASE 4: INSOLVENCY PROOF & ASSET SEIZURE
    // ============================================================

    fn get_mint_decimals(mint_info: &AccountInfo) -> Result<u8, ProgramError> {
        let data = mint_info.try_borrow_data()?;
        let mint = Mint::unpack(&data).map_err(|_| {
            ProgramError::from(OrchestratorError::InvalidAccountData.into_program_error())
        })?;
        Ok(mint.decimals)
    }

    let collateral_decimals = get_mint_decimals(collateral_mint_info)?;
    let debt_decimals = get_mint_decimals(debt_mint_info)?;

    // প্রাইসকে I80F48 ফরম্যাটে রূপান্তর
    let price_fp: i128 = {
        let raw_price = sanitized_price as i128;
        let shift = 48i32;
        let expo = sanitized_expo;
        let ten_pow = |exp: u32| -> Result<i128, ProgramError> {
            10i128.checked_pow(exp).ok_or_else(|| ProgramError::from(
                OrchestratorError::MathOverflow.into_program_error()
            ))
        };

        if expo >= 0 {
            let scale = ten_pow(expo as u32)?;
            raw_price
                .checked_mul(scale)
                .and_then(|v| v.checked_shl(shift as u32))
                .ok_or_else(|| ProgramError::from(
                    OrchestratorError::MathOverflow.into_program_error()
                ))?
        } else {
            let scale = ten_pow(expo.unsigned_abs())?;
            raw_price
                .checked_shl(shift as u32)
                .and_then(|v| v.checked_div(scale))
                .ok_or_else(|| ProgramError::from(
                    OrchestratorError::MathOverflow.into_program_error()
                ))?
        }
    };

    // কোলাটারালের ফিয়াট মান
    let collateral_raw = vault_state.total_collateral as i128;
    let collateral_dec_scale = 10i128
        .checked_pow(collateral_decimals as u32)
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;
    let collateral_fiat_fp = collateral_raw
        .checked_mul(price_fp)
        .and_then(|v| v.checked_div(collateral_dec_scale))
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;

    // ঋণের ফিয়াট মান
    let debt_raw = vault_state.total_debt_shares as i128;
    // ফিক্স ১: i128::from_le_bytes ব্যবহার
    let acc_interest_bits = i128::from_le_bytes(vault_state.accumulated_interest_index.bits);
    let debt_dec_scale = 10i128
        .checked_pow(debt_decimals as u32)
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;
    let debt_fp = debt_raw
        .checked_mul(acc_interest_bits)
        .and_then(|v| v.checked_shr(48)) // 2^48 দিয়ে ভাগ
        .and_then(|v| v.checked_div(debt_dec_scale))
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;

    if debt_fp == 0 {
        return Err(ProgramError::from(
            OrchestratorError::VaultHealthy.into_program_error(),
        ));
    }

    // হেলথ রেশিও
    let health_ratio_fp = collateral_fiat_fp
        .checked_div(debt_fp)
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;
    let min_ratio_bits = i128::from_le_bytes(vault_state.min_collateral_ratio.bits);
    if health_ratio_fp >= min_ratio_bits {
        return Err(ProgramError::from(
            OrchestratorError::VaultHealthy.into_program_error(),
        ));
    }

    // পেলোড থেকে repay_debt_shares
    if payload.len() < 8 {
        return Err(ProgramError::from(
            OrchestratorError::InvalidInstructionData.into_program_error(),
        ));
    }
    let repay_debt_shares = u64::from_le_bytes(
        payload[0..8].try_into().map_err(|_| {
            ProgramError::from(OrchestratorError::InvalidInstructionData.into_program_error())
        })?,
    ) as i128;

    // ঋণ পরিশোধের ফিয়াট মান
    let repay_debt_fp = repay_debt_shares
        .checked_shl(48)
        .and_then(|v| v.checked_div(debt_dec_scale))
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;

    // লিকুইডেশন বোনাস
    let bonus_bps = vault_state.jit_fee_bps as i128;
    let seize_value_fp = repay_debt_fp
        .checked_mul(10_000i128 + bonus_bps)
        .and_then(|v| v.checked_div(10_000))
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;

    // কোলাটারালের পরিমাণ (raw units)
    let collateral_to_seize_raw = seize_value_fp
        .checked_mul(collateral_dec_scale)
        .and_then(|v| v.checked_div(price_fp))
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;

    if collateral_to_seize_raw > vault_state.total_collateral as i128 {
        return Err(ProgramError::from(
            OrchestratorError::CollateralRatioTooLow.into_program_error(),
        ));
    }

    // ============================================================
    // PHASE 5: PHYSICAL EXECUTION (CPI)
    // ============================================================
    // ফিক্স ৪: বার্নের জন্য invoke ব্যবহার (কিপার সাধারণ সাইনার)
    let burn_ix = burn(
        token_program_info.key,
        keeper_debt_ata_info.key,
        debt_mint_info.key,
        keeper_info.key,
        &[],
        repay_debt_shares as u64,
    )?;
    invoke(
        &burn_ix,
        &[
            keeper_debt_ata_info.clone(),
            debt_mint_info.clone(),
            keeper_info.clone(),
            token_program_info.clone(),
        ],
    )?;

    // ভল্ট PDA-র জন্য invoke_signed
    let vault_seeds: &[&[u8]] = &[
        b"vault",
        collateral_mint_info.key.as_ref(),
        vault_state.authority.as_ref(),
        &[vault_state.bump],
    ];
    let transfer_ix = transfer(
        token_program_info.key,
        vault_collateral_ata_info.key,
        keeper_collateral_ata_info.key,
        vault_info.key,
        &[],
        collateral_to_seize_raw as u64,
    )?;
    invoke_signed(
        &transfer_ix,
        &[
            vault_collateral_ata_info.clone(),
            keeper_collateral_ata_info.clone(),
            vault_info.clone(),
            token_program_info.clone(),
        ],
        &[vault_seeds],
    )?;

    // ============================================================
    // PHASE 6: STATE MUTATION
    // ============================================================
    vault_state.total_collateral = vault_state
        .total_collateral
        .checked_sub(collateral_to_seize_raw as u64)
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;
    vault_state.total_debt_shares = vault_state
        .total_debt_shares
        .checked_sub(repay_debt_shares as u64)
        .ok_or_else(|| ProgramError::from(
            OrchestratorError::MathOverflow.into_program_error()
        ))?;

    Ok(())
}