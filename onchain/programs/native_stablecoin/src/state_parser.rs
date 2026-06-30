pub fn parse_state<T>(_bytes: &[u8]) -> &T {
    unsafe { &*(_bytes.as_ptr() as *const T) }
}
