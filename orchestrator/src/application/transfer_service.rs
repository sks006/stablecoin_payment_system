use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use shared_memory::OrchestratorError;

pub struct TransactionBuilder {
    program_id: Pubkey,
}

impl TransactionBuilder {
    pub fn new(program_id: Pubkey) -> Self {
        Self { program_id }
    }

    /// Constructs the raw Sealevel instruction for Sub-Millisecond JIT Minting
    pub fn build_mint_jit_ix(
        &self,
        funder_wallet: Pubkey,
        funder_token: Pubkey,
        vault: Pubkey,
        vault_token: Pubkey,
        collateral_mint: Pubkey,
        debt_amount: u64,
        collateral_deposit: u64,
    ) -> Result<Instruction, OrchestratorError> {
        
        // === 1. PAYLOAD ALLOCATION ===
        let mut data = Vec::with_capacity(17);
        data.push(0u8); // Discriminator for MintJit
        data.extend_from_slice(&debt_amount.to_le_bytes());
        data.extend_from_slice(&collateral_deposit.to_le_bytes());
        
        // === 2. ACCOUNT META ORDERING ===
        let accounts = vec![
            AccountMeta::new(funder_wallet, true),
            AccountMeta::new(funder_token, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(vault_token, false),
            AccountMeta::new_readonly(collateral_mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];
        
        // === 3. INSTRUCTION ASSEMBLY ===
        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }
}