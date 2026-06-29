use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    transaction::Transaction,
};

pub struct TransactionBuilder;

impl TransactionBuilder {
    pub fn build_mint_jit_tx(
        vault: Pubkey,
        owner: Pubkey,
        program_id: Pubkey,
        amount: u64,
        nonce: u64,
    ) -> Transaction {
        let mut data = Vec::with_capacity(17);
        data.push(0); // MintJit instruction tag
        data.extend_from_slice(&amount.to_le_bytes());
        data.extend_from_slice(&nonce.to_le_bytes());

        let inst = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(owner, true),
            ],
            data,
        };

        Transaction::new_with_payer(&[inst], Some(&owner))
    }
}
