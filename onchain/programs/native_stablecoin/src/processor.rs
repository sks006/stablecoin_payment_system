// native_stablecoin/src/processor.rs

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::ProgramError,
};
use crate::instructions::{mint_jit, liquidate, settle};
use shared_memory::error::{OrchestratorError, ErrorConversion};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (tag, payload) = instruction_data.split_first().ok_or_else(|| {
        ProgramError::from(OrchestratorError::InvalidInstructionData.into_program_error())
    })?;

    match tag {
        0 => mint_jit::process(program_id, accounts, payload),
        1 => liquidate::process(program_id, accounts, payload),
        2 => settle::process(program_id, accounts, payload),
        _ => Err(ProgramError::from(
            OrchestratorError::InvalidInstructionData.into_program_error(),
        )),
    }
}
