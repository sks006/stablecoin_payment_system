use anchor_lang::prelude::*;
use shared_memory::state::UserVaultState;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<UserVaultState>(),
        seeds = [b"vault", owner.key().as_ref()],
        bump
    )]
    pub vault: AccountLoader<'info, UserVaultState>,
    pub owner: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeVault>, bump: u8) -> Result<()> {
    let mut vault = ctx.accounts.vault.load_init()?;
    vault.version = 1;
    vault.bump = bump;
    vault.owner = ctx.accounts.owner.key();
    vault.primary_delegate = Pubkey::default();
    vault.governance_authority = ctx.accounts.payer.key();
    vault.collateral_balance = 0;
    vault.debt_balance = 0;
    vault.state_flags = 0;
    vault.reserved_buffer = [0; 256];
    Ok(())
}
