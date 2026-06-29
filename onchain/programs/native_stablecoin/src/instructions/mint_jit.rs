use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::ProgramError,
};
use shared_memory::instructions::MintJitPayload;
use crate::state_parser::StateParser;

pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    payload: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let vault_info = next_account_info(accounts_iter)?;
    let authority_info = next_account_info(accounts_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let payload: &MintJitPayload = bytemuck::try_from_bytes(payload)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    let vault = StateParser::parse_vault_mut(vault_info)?;
    
    vault.collateral_balance = vault.collateral_balance
        .checked_add(payload.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
        
    vault.debt_balance = vault.debt_balance
        .checked_add(payload.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
