#[cfg(feature = "anchor-bridge")]
#[repr(C)]
pub struct VaultState {
    pub authority: [u8; 32],
    pub bump: u8,
    pub reserved: [u8; 31],
}

#[cfg(feature = "native-bridge")]
#[repr(C)]
pub struct VaultState {
    pub authority: [u8; 32],
    pub bump: u8,
    pub reserved: [u8; 31],
}
