use anchor_lang::prelude::*;

#[event]
pub struct VaultInitialized {
    pub vault: Pubkey,
    pub owner: Pubkey,
}

#[event]
pub struct AdminConfigured {
    pub vault: Pubkey,
    pub threshold: u64,
    pub fee_rate: u16,
}
