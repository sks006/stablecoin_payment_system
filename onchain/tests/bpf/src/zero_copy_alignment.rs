#[cfg(test)]
mod tests {
    use shared_memory::state::JitVaultState;
    use std::mem::size_of;

    #[test]
    fn verify_jit_vault_state_layout() {
        // Enforce the size is exactly 440 bytes
        assert_eq!(size_of::<JitVaultState>(), 440, "JitVaultState must be exactly 440 bytes");

        // Enforce alignment is 8 bytes
        assert_eq!(std::mem::align_of::<JitVaultState>(), 8, "JitVaultState must be 8-byte aligned");

        // Dynamically compute offsets to verify no compiler padding is injected
        let state: JitVaultState = unsafe { std::mem::zeroed() };
        let base = &state as *const _ as usize;

        let offset_discriminator = (&state.discriminator as *const _ as usize) - base;
        let offset_authority = (&state.authority as *const _ as usize) - base;
        let offset_collateral_mint = (&state.collateral_mint as *const _ as usize) - base;
        let offset_debt_mint = (&state.debt_mint as *const _ as usize) - base;
        let offset_total_collateral = (&state.total_collateral as *const _ as usize) - base;
        let offset_total_debt_shares = (&state.total_debt_shares as *const _ as usize) - base;
        let offset_accumulated_interest = (&state.accumulated_interest_index as *const _ as usize) - base;
        let offset_jit_liquidation_window = (&state.jit_liquidation_window as *const _ as usize) - base;
        let offset_min_collateral_ratio = (&state.min_collateral_ratio as *const _ as usize) - base;
        let offset_target_collateral_ratio = (&state.target_collateral_ratio as *const _ as usize) - base;
        let offset_jit_fee_bps = (&state.jit_fee_bps as *const _ as usize) - base;
        let offset_bump = (&state.bump as *const _ as usize) - base;
        let offset_reserved = (&state.reserved as *const _ as usize) - base;
        let offset_padding = (&state._padding as *const _ as usize) - base;

        assert_eq!(offset_discriminator, 0);
        assert_eq!(offset_authority, 8);
        assert_eq!(offset_collateral_mint, 40);
        assert_eq!(offset_debt_mint, 72);
        assert_eq!(offset_total_collateral, 104);
        assert_eq!(offset_total_debt_shares, 112);
        assert_eq!(offset_accumulated_interest, 120);
        assert_eq!(offset_jit_liquidation_window, 136);
        assert_eq!(offset_min_collateral_ratio, 144);
        assert_eq!(offset_target_collateral_ratio, 160);
        assert_eq!(offset_jit_fee_bps, 176);
        assert_eq!(offset_bump, 178);
        assert_eq!(offset_reserved, 179);
        assert_eq!(offset_padding, 435);
    }
}
