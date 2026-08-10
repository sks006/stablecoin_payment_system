use anchor_lang::prelude::*;

declare_id!("7aM25wz7W4pM3LrdHj59eSwxH517eWcKqJ7T38wFkY1c");

#[path = "instructions/mod.rs"]
pub mod handlers;
pub mod events;

pub use handlers::initialize::InitializeVault;
pub(crate) use handlers::initialize::__client_accounts_initialize_vault;
pub use handlers::admin::SetAdminConfig;
pub(crate) use handlers::admin::__client_accounts_set_admin_config;

#[program]
pub mod anchor_stablecoin {
    use super::*;

    pub fn initialize(ctx: Context<InitializeVault>) -> Result<()> {
        handlers::initialize::process_initialize(ctx)
    }

    pub fn set_admin_config(
        ctx: Context<SetAdminConfig>,
        threshold: u64,
        fee: u64
    ) -> Result<()> {
        handlers::admin::set_admin_config(ctx, threshold, fee)
    }
}

#[event]
pub struct VaultInitialized {}
