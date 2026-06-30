use anchor_lang::prelude::*;

pub fn set_admin_config(_ctx: Context<SetAdminConfig>, _threshold: u64, _fee: u64) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct SetAdminConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
