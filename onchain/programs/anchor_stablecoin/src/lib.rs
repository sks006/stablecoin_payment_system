use anchor_lang::prelude::*;
use shared_memory::state::UserVaultState;

pub mod instructions;
pub mod events;

use instructions::*;

declare_id("CPlane1111111111111111111111111111111111111");

#[program]
pub mod anchor_stablecoin {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>, bump: u8) -> Result<()> {
        instructions::initialize::handler(ctx, bump)
    }

    pub fn configure_admin(ctx: Context<ConfigureAdmin>, threshold: u64, fee_rate: u16) -> Result<()> {
        instructions::admin::handler(ctx, threshold, fee_rate)
    }
}

impl anchor_lang::Owner for UserVaultState {
    fn owner() -> Pubkey {
        crate::ID
    }
}

impl anchor_lang::ZeroCopy for UserVaultState {}

impl anchor_lang::Discriminator for UserVaultState {
    const DISCRIMINATOR: [u8; 8] = [117, 115, 101, 114, 118, 97, 117, 108]; // "uservaul"
    fn discriminator() -> [u8; 8] {
        Self::DISCRIMINATOR
    }
}
