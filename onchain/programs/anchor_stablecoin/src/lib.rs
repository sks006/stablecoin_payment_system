use anchor_lang::prelude::*;

pub mod instructions;
pub mod events;

#[program]
pub mod anchor_stablecoin {
    use super::*;

    pub fn initialize(ctx: Context<instructions::initialize::InitializeVault>) -> Result<()> {
        instructions::initialize::initialize_vault(ctx)
    }

    pub fn set_admin_config(
        ctx: Context<instructions::admin::SetAdminConfig>,
        threshold: u64,
        fee: u64
    ) -> Result<()> {
        instructions::admin::set_admin_config(ctx, threshold, fee)
    }
}

#[event]
pub struct VaultInitialized {}
