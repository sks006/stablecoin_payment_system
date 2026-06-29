use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey,
    program_error::ProgramError,
};
use crate::instructions::{mint_jit, liquidate, settle};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        if instruction_data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let instruction_type = instruction_data[0];
        let payload = &instruction_data[1..];

        match instruction_type {
            0 => mint_jit::process(program_id, accounts, payload),
            1 => liquidate::process(program_id, accounts, payload),
            2 => settle::process(program_id, accounts, payload),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
