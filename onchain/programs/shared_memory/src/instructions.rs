// shared_memory/src/instructions.rs

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeInstruction {
    MintJit = 0,
    Liquidate = 1,
    Settle = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
// Enforce bytemuck::Pod and bytemuck::Zeroable traits
pub struct MintJitPayload {
    pub instruction_id: u8,       // 1 byte (Must map to NativeInstruction::MintJit)
    pub amount: u64,              // 8 bytes
    pub expected_fee_bps: u16,    // 2 bytes
    pub _padding: [u8; 5],        // 5 bytes: Aligns 11 bytes up to 16
}

#[repr(C)]
#[derive(Copy, Clone)]
// Enforce bytemuck::Pod and bytemuck::Zeroable traits
pub struct LiquidatePayload {
    pub instruction_id: u8,       // 1 byte
    pub target_debt_shares: u64,  // 8 bytes
    pub slippage_tolerance: u64,  // 8 bytes
    pub _padding: [u8; 7],        // 7 bytes: Aligns 17 bytes up to 24
}