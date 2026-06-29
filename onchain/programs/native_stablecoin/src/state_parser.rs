use solana_program::{account_info::AccountInfo, program_error::ProgramError};
use shared_memory::state::UserVaultState;

pub struct StateParser;

impl StateParser {
    pub fn parse_vault_mut<'a>(
        account_info: &AccountInfo<'a>,
    ) -> Result<&'a mut UserVaultState, ProgramError> {
        let data = account_info.try_borrow_mut_data()?;
        if data.len() < 8 + std::mem::size_of::<UserVaultState>() {
            return Err(ProgramError::InvalidAccountData);
        }
        let (_, body) = data.split_at_mut(8);
        let state: &mut UserVaultState = bytemuck::try_from_bytes_mut(body)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(state)
    }

    pub fn parse_vault<'a>(
        account_info: &AccountInfo<'a>,
    ) -> Result<&'a UserVaultState, ProgramError> {
        let data = account_info.try_borrow_data()?;
        if data.len() < 8 + std::mem::size_of::<UserVaultState>() {
            return Err(ProgramError::InvalidAccountData);
        }
        let (_, body) = data.split_at(8);
        let state: &UserVaultState = bytemuck::try_from_bytes(body)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(state)
    }
}
