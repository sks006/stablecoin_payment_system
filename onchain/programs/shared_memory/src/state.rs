#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FixedPoint {
    // 16 bytes: Universally aligned storage for i128.
    // Use i128::from_le_bytes(self.bits) in your math logic.
    pub bits: [u8; 16], 
}

impl FixedPoint {
    pub fn from_bits(bits: i128) -> Self {
        Self {
            bits: bits.to_le_bytes(),
        }
    }

    pub fn to_bits(self) -> i128 {
        i128::from_le_bytes(self.bits)
    }
}

pub const VAULT_STATE_DISCRIMINATOR: [u8; 8] = [0x7e, 0xe4, 0xa1, 0x80, 0xbe, 0x06, 0xd6, 0xbb];


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct JitVaultState {
    // === SYSTEM ABI ===
    pub discriminator: [u8; 8],           // 8 bytes: Explicit offset for native parsing

    // === AUTHORITY & IDENTIFICATION ===
    pub authority: [u8; 32],              // 32 bytes
    pub collateral_mint: [u8; 32],        // 32 bytes
    pub debt_mint: [u8; 32],              // 32 bytes

    // === VAULT BALANCES ===
    pub total_collateral: u64,            // 8 bytes
    pub total_debt_shares: u64,           // 8 bytes

    // === JIT CONFIGURATION & RISK ===
    pub accumulated_interest_index: FixedPoint, // 16 bytes: Replaces u128
    pub jit_liquidation_window: u64,            // 8 bytes
    pub min_collateral_ratio: FixedPoint,       // 16 bytes: Replaces u32
    pub target_collateral_ratio: FixedPoint,    // 16 bytes: Replaces u32
    pub jit_fee_bps: u16,                       // 2 bytes

    // === STATE TRACKING ===
    pub bump: u8,                         // 1 byte

    // === SECURE UPGRADEABILITY ===
    pub reserved: [u8; 256],              // 256 bytes: Forward-compatibility buffer
    
    // === MEMORY ALIGNMENT ===
    pub _padding: [u8; 5],                // 5 bytes: Aligns 435 bytes up to 440 (multiple of 8)
}




