unsafe extern "C" {
    /// Calls the Go implementation of add_u32.
    /// Signature matches the generated C header: uint32_t add_u32(uint32_t, uint32_t)
    fn add_u32(a: u32, b: u32) -> u32;
}

/// Adds two `u32` values using the Go static library.
pub fn go_add_u32(a: u32, b: u32) -> u32 {
    // SAFETY: add_u32 is a pure arithmetic function with no side-effects or aliasing.
    unsafe { add_u32(a, b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_addition() {
        assert_eq!(go_add_u32(2, 3), 5);
    }

    #[test]
    fn test_zero() {
        assert_eq!(go_add_u32(0, 0), 0);
    }

    #[test]
    fn test_max_boundary() {
        assert_eq!(go_add_u32(u32::MAX - 1, 1), u32::MAX);
    }
}
