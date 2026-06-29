#[cfg(test)]
mod tests {
    use shared_memory::state::UserVaultState;
    use std::mem::align_of;

    #[test]
    fn test_alignment() {
        assert_eq!(align_of::<UserVaultState>(), 8, "UserVaultState must be 8-byte aligned");
        assert_eq!(std::mem::size_of::<UserVaultState>(), 384, "UserVaultState must be exactly 384 bytes");
    }
}
