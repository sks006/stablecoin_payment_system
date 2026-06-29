use anchor_lang::prelude::*;
use shared_memory::state::UserVaultState;

#[derive(Accounts)]
pub struct ConfigureAdmin<'info> {
    #[account(mut, has_one = governance_authority)]
    pub vault: AccountLoader<'info, UserVaultState>,
    pub governance_authority: Signer<'info>,
}

pub fn handler(ctx: Context<ConfigureAdmin>, threshold: u64, fee_rate: u16) -> Result<()> {
    let mut vault = ctx.accounts.vault.load_mut()?;
    vault.reserved_buffer[0..8].copy_from_slice(&threshold.to_le_bytes());
    vault.reserved_buffer[8..10].copy_from_slice(&fee_rate.to_le_bytes());
    Ok(())
}
