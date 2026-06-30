use anchor_lang::prelude::*;

pub fn initialize_vault(_ctx: Context<InitializeVault>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
