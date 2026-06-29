use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserVaultState {
    pub version: u8,                  // Byte 0
    pub bump: u8,                     // Byte 1
    pub _alignment_padding0: [u8; 6], // Bytes 2-7
    pub owner: Pubkey,                // Bytes 8-39
    pub primary_delegate: Pubkey,     // Bytes 40-71
    pub governance_authority: Pubkey, // Bytes 72-103
    pub collateral_balance: u64,      // Bytes 104-111
    pub debt_balance: u64,            // Bytes 112-119
    pub state_flags: u64,             // Bytes 120-127
    pub reserved_buffer: [u8; 256],   // Bytes 128-383
}

unsafe impl bytemuck::Zeroable for UserVaultState {}
unsafe impl bytemuck::Pod for UserVaultState {}
