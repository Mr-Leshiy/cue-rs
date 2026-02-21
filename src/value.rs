use std::ffi::{CString, NulError};
use std::os::raw::c_char;

use crate::error::CueError;

/// Opaque handle to a compiled CUE value managed by the Go runtime.
/// Obtain one with [`go_cue_value_new`] and release it with [`go_cue_value_free`].
type CueValue = usize;

unsafe extern "C" {
    fn cue_value_new(input: *const c_char) -> CueValue;
    fn cue_value_free(handle: CueValue);
        /// Returns a malloc-allocated C string; caller must free it.
    fn cue_value_validate(handle: CueValue) -> *mut c_char;
}

pub struct Value(CueValue);

impl Drop for Value {
    /// Releases the Go-side storage associated with `handle`.
    fn drop(&mut self) {
        // SAFETY: handle was produced by go_cue_value_new and has not been freed.
        unsafe { cue_value_free(self.0) }
    }
}

impl Value {
    /// Compiles `input` as a CUE expression and returns an opaque [`Value`] handle.
    ///
    /// # Errors:
    /// This function will return an error if the supplied string contains an
    /// internal 0 byte. The [`NulError`] returned will contain the bytes as well as
    /// the position of the nul byte.
    pub fn new(s: &str) -> Result<Self, NulError> {
        let c_str = CString::new(s)?;
        Ok(Value(unsafe { cue_value_new(c_str.as_ptr()) }))
    }

    /// Validates the CUE value and returns any error message.
    ///
    /// Returns `Ok(())` when the value is valid.
    /// 
    /// Errors:
    /// 
    pub fn validate(&self) -> Result<(), CueError> {
        // SAFETY: cue_value_validate returns a pointer from C.CString (malloc).
        // CString::from_raw takes ownership and calls free when dropped.
        let ptr = unsafe { cue_value_validate(self.0) };
        if ptr.is_null() {
            return Err(CueError::InvalidValuePointerAddress);
        }
        let c_str = unsafe { CString::from_raw(ptr) };
        if c_str.is_empty() {
            Ok(())
        } else {
            Err(CueError::ValidationError(c_str.to_string_lossy().into_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_cue_value() {
        let value = Value::new(r#"{ name: "alice", age: 30 }"#).unwrap();
        assert!(value.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_cue_value() {
        // Unclosed brace is a syntax error; validate must return an error string.
        let value = Value::new("{ name: ").unwrap();
        assert!(value.validate().is_err());
    }
}
