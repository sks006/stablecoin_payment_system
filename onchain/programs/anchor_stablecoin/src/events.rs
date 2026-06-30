use anchor_lang::prelude::*;

#[event]
pub struct VaultInitialized {
    pub authority: Pubkey,
}
