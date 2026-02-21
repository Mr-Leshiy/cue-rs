use std::ffi::CString;
use std::os::raw::c_char;

unsafe extern "C" {
    /// Calls the Go implementation of validate.
    /// Compiles the input as a CUE Value and returns whether it is valid.
    fn validate(input: *const c_char) -> bool;
}

/// Validates a string as a CUE Value using `cuelang.org/go/cue`.
///
/// Returns `false` if the string contains interior null bytes, or if the
/// input cannot be compiled into a valid CUE Value.
pub fn go_validate(input: &str) -> bool {
    let Ok(c_str) = CString::new(input) else {
        return false;
    };
    // SAFETY: validate is a read-only function; the pointer is valid for the duration of the call.
    unsafe { validate(c_str.as_ptr()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_cue_value() {
        // A well-formed CUE struct compiles into a valid Value.
        assert!(go_validate(r#"{ name: "alice", age: 30 }"#));
    }

    #[test]
    fn test_validate_invalid_cue_value() {
        // Unclosed brace is a syntax error; CompileString will set Err().
        assert!(!go_validate("{ name: "));
    }
}
