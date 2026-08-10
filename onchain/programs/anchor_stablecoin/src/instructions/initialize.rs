use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke_signed,
    rent::Rent,
    sysvar::Sysvar,
    system_instruction,
};
use shared_memory::{
    error::OrchestratorError,
    state::{FixedPoint, JitVaultState, VAULT_STATE_DISCRIMINATOR},
};


#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Used only for PDA derivation; no Anchor deserialization.
    pub collateral_mint: AccountInfo<'info>,

    /// CHECK: Allocated and validated manually via CPI.
    #[account(mut)]
    pub vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn process_initialize(ctx: Context<InitializeVault>) -> Result<()> {
    // ---------------------------------------------------------
    // STEP 1: PDA Verification
    // ---------------------------------------------------------
    let (expected_vault, bump) = Pubkey::find_program_address(
        &[
            b"vault",
            ctx.accounts.collateral_mint.key().as_ref(),
            ctx.accounts.admin.key().as_ref(),
        ],
        &native_stablecoin::ID,
    );

    if ctx.accounts.vault.key() != expected_vault {
        return Err(OrchestratorError::InvalidPDA.into());
    }

    // ---------------------------------------------------------
    // STEP 2: Rent Exemption Calculation
    // ---------------------------------------------------------
    let rent = Rent::get()?;
    let vault_size = std::mem::size_of::<JitVaultState>();   // 440, always in sync
    let lamports = rent.minimum_balance(vault_size);

    // ---------------------------------------------------------
    // STEP 3: Build the CreateAccount instruction
    // ---------------------------------------------------------
    let create_ix = system_instruction::create_account(
        ctx.accounts.admin.key,
        ctx.accounts.vault.key,
        lamports,
        vault_size as u64,
        &native_stablecoin::ID,
    );

    // ---------------------------------------------------------
    // STEP 4: Execute allocation (invoke_signed for PDA)
    // ---------------------------------------------------------
    // The seeds must include the exact bump used during derivation.
    let collateral_mint_key = ctx.accounts.collateral_mint.key();
    let admin_key = ctx.accounts.admin.key();
    let seeds: &[&[u8]] = &[
        b"vault",
        collateral_mint_key.as_ref(),
        admin_key.as_ref(),
        &[bump],
    ];

    invoke_signed(
        &create_ix,
        &[
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[seeds],
    )?;

    // ---------------------------------------------------------
    // STEP 5: Zero‑Copy Hydration
    // ---------------------------------------------------------
    let mut buffer = ctx.accounts.vault.try_borrow_mut_data()?;
    let vault_state: &mut JitVaultState = bytemuck::try_from_bytes_mut(&mut buffer)
        .map_err(|_| OrchestratorError::UnalignedMemoryAccess)?;

    // Discriminator – must match native program expectation
    vault_state.discriminator = VAULT_STATE_DISCRIMINATOR;

    // Authority & asset identification
    vault_state.authority = ctx.accounts.admin.key().to_bytes();
    vault_state.collateral_mint = ctx.accounts.collateral_mint.key().to_bytes();
    vault_state.debt_mint = Pubkey::default().to_bytes();   // to be set later

    // Balances
    vault_state.total_collateral = 0;
    vault_state.total_debt_shares = 0;

    // JIT configuration (fixed‑point using integer arithmetic)
    vault_state.accumulated_interest_index = FixedPoint::from_bits((1u128 << 48) as i128); // 1.0
    vault_state.jit_liquidation_window = 3600;  // seconds
    vault_state.min_collateral_ratio = FixedPoint::from_bits((12 * (1u128 << 48) / 10) as i128); // 1.2
    vault_state.target_collateral_ratio = FixedPoint::from_bits((16 * (1u128 << 48) / 10) as i128); // 1.6
    vault_state.jit_fee_bps = 10;

    // PDA bump for native program signing
    vault_state.bump = bump;

    // Forward‑compatibility buffer & alignment
    vault_state.reserved = [0u8; 256];
    vault_state._padding = [0u8; 5];

    Ok(())
}