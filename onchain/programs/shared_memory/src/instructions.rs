#[cfg(feature = "anchor-bridge")]
pub const INIT_VAULT: &[u8] = b"init_vault";

#[cfg(feature = "anchor-bridge")]
pub const SET_ADMIN_CONFIG: &[u8] = b"set_admin_config";

#[cfg(feature = "native-bridge")]
pub const MINT_JIT: &[u8] = b"mint_jit";

#[cfg(feature = "native-bridge")]
pub const LIQUIDATE: &[u8] = b"liquidate";

#[cfg(feature = "native-bridge")]
pub const SETTLE: &[u8] = b"settle";
