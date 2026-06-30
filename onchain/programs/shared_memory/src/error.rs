#[cfg_attr(feature = "anchor-bridge", derive(anchor_lang::Discriminator))]
pub enum SharedError {
    InvalidAuthority = 6000,
    InvalidConfig = 6001,
    InvalidInstruction = 6002,
}
