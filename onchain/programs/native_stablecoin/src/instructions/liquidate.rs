use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::ProgramError,
};
use shared_memory::instructions::LiquidatePayload;
use crate::state_parser::StateParser;

pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    payload: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let vault_info = next_account_info(accounts_iter)?;
    let liquidator_info = next_account_info(accounts_iter)?;

    if !liquidator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let payload: &LiquidatePayload = bytemuck::try_from_bytes(payload)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let vault = StateParser::parse_vault_mut(vault_info)?;

    let debt_to_cover = std::cmp::min(vault.debt_balance, payload.max_debt_to_cover);
    
    vault.debt_balance = vault.debt_balance
        .checked_sub(debt_to_cover)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let collateral_to_claim = debt_to_cover;
    vault.collateral_balance = vault.collateral_balance
        .checked_sub(collateral_to_claim)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
