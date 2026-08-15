use share_memory::error::{OrchestratorError,ErrorConversion};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    pubkey::Pubkey,

}

use spl_token::instruction::transfer;
use crate::state_parsers::load_mut_vault_state;

pub fn process(program_id:&Pubkey, accounts:&[AccountInfo],payload:&[u8])-> ProgramResult{
    
 let accounts_iter = &mut accounts.iter();
 let funder_wallet_info= next_account_info(account_info)?;
 let funder_token_info=next_account_info(account_info)?;
 let vault_info=next_account_info(account_info)?;
 let vault_token_info=next_account_info(account_info)?;
 let collateral_mint_info=next_account_info(account_info)?;
 let token_program_info=next_account_info(account_info)?;
 
 //Signer checks
 
 if !funder_wallet_info.is_signer{
    return Err(OrchestratorError::AccountNotSigner("funder_wallet").to_program_error());
 }
//2 hardware validation (zero-copy lock)
 let vault_state=load_mut_vault_state(vault_info,program_id)?;
 //3 Collateral mint binding (prevents fake-token attacks)
 if collateral_mint_info.key !=vault_state.collateral_mint{
    return Err(
        ProgramResult::from(OrchestratorError::CollateralMintMismatch.to_program_error())
    )
 }
 //4 Bounds check (prevents panic on short payload)
 if payload.len()<16{
    return Err(OrchestratorError::InvalidInstructionData.to_program_error());
 }
 //5 safe deserialization (error wrapsinto programError)

 let (debt_byte,collateral_bytes)=payload.split_at(8)
debt_bytes.try_into().map_err(|_| OrchestratorError::InvalidInstructionData.to_program_error())? 

let requested_collateral_deposit=u64::from_le_bytes(collateral_bytes.try_into().map_err(|_|{
    programError::InvalidInstructionData.into_program_error()
})?);

//6 Mathematical risk boundary (fixed-point)------

let pre_collateral:i128= vault_state.total_collateral.into();

let pre_debt:i128=vault_state.total_debt_share();

let post_collateral=pre_collateral
.checked_add(requested_collateral_deposit as i128).ok_or_else(||OrchestratorError::MathOverflow.to_program_error())?;

let post_debt=pre_debt.checked_add(debt_amount as i128).ok_or_else(||OrchestratorError::MathOverflow.to_program_error())?;

if post_debt==0{
    return Err(programError::from(OrchestratorError::MathOverflow.into_program_error()));
}

let collateral_scaler= post_collateral.checked_shl(48)
.ok_or_else(||{
    programError::MathOverflow.into_program_error()
})?;

let new_ratio_bits=collateral_scaled.checked_div(post_debt)
.ok_or_else(||{
    programError::MathOverflow.into_program_error()
})?;

let min_ratio_bits=i128::from_le_bytes(vault_state.min_collateral_ratio.bits);
if new_ratio_bits<min_ratio_bits{
    return Err(programError::from(
        OrchestratorError::CollateralRationTooLow.into_program_error()
    ))   
}
  
  // 7 physical token transfer (CPI actual token movement)

  let transfer_ix=transfer(
    token_program_info.key,
    funder_token_info.key, //source ATA
    vault_token_info.key, //destination AT
    funder_wallet_info.key, // owner
    &[funder_wallet_info.key] // signer
    requested_collateral_deposit
  )?;

  invoke(
    &transfer_ix,
    &[
        token_program_info.clone(),
        funder_token_info.clone(),
        vault_token_info.clone(),
        funder_wallet_info.clone(),
    ]
  )?;

    // ---- 8. State mutation (only after tokens are moved) ----
    vault_state.total_collateral = post_collateral as u64;
    vault_state.total_debt_shares = post_debt as u64;

    Ok(())


}