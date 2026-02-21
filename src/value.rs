//! CUE value type and associated operations.

use std::{
    ffi::{CString, NulError},
    os::raw::c_char,
};

use crate::error::CueError;

/// Opaque handle to a compiled CUE value managed by the Go runtime.
/// Obtain one with [`cue_value_new`] and release it with [`cue_value_free`].
type CueValueAddr = usize;

unsafe extern "C" {
    /// Compiles `input` as a CUE expression and returns an opaque handle.
    fn cue_value_new(input: *const c_char) -> CueValueAddr;
    /// Releases the Go-side storage for the given handle.
    fn cue_value_free(addr: CueValueAddr);
    /// Returns the unification of two CUE values as a new opaque handle.
    fn cue_value_unify(
        addr1: CueValueAddr,
        addr2: CueValueAddr,
    ) -> CueValueAddr;
    /// Returns a malloc-allocated C string; caller must free it.
    fn cue_value_validate(addr: CueValueAddr) -> *mut c_char;
    /// Returns a malloc-allocated JSON string, or NULL on error; caller must free it.
    fn cue_value_to_json(addr: CueValueAddr) -> *mut c_char;
    /// Returns a malloc-allocated YAML string, or NULL on error; caller must free it.
    fn cue_value_to_yaml(addr: CueValueAddr) -> *mut c_char;
}

/// A compiled CUE value, backed by Go runtime storage.
pub struct Value(CueValueAddr);

impl Drop for Value {
    /// Releases the Go-side storage associated with this value.
    fn drop(&mut self) {
        unsafe { cue_value_free(self.0) }
    }
}

impl Value {
    /// Compiles `input` as a CUE expression and returns a [`Value`].
    ///
    /// # Errors
    ///
    /// Returns a [`NulError`] if the supplied string contains an internal
    /// null byte; the error includes the bytes and position of the null byte.
    pub fn new(s: &str) -> Result<Self, NulError> {
        let c_str = CString::new(s)?;
        Ok(Self(unsafe { cue_value_new(c_str.as_ptr()) }))
    }

    /// Computes the unification of two CUE values.
    ///
    /// <https://pkg.go.dev/cuelang.org/go/cue#Value.Unify>
    #[must_use]
    pub fn unify(
        val1: &Value,
        val2: &Value,
    ) -> Value {
        Self(unsafe { cue_value_unify(val1.0, val2.0) })
    }

    /// Validates the CUE value against its constraints.
    ///
    /// <https://pkg.go.dev/cuelang.org/go/cue#Value.Validate>
    ///
    /// # Errors
    ///
    /// Returns [`CueError::InvalidValuePointerAddress`] if the underlying C
    /// function returns a null pointer, or [`CueError::ValidationError`] if
    /// the value violates its constraints.
    pub fn validate(&self) -> Result<(), CueError> {
        let ptr = unsafe { cue_value_validate(self.0) };
        if ptr.is_null() {
            return Err(CueError::InvalidValuePointerAddress);
        }
        let c_str = unsafe { CString::from_raw(ptr) };
        if c_str.is_empty() {
            Ok(())
        } else {
            Err(CueError::ValidationError(
                c_str.to_string_lossy().into_owned(),
            ))
        }
    }

    /// Encodes the CUE value as a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails (see [`Value::validate`]) or if
    /// the underlying C function returns a null pointer.
    pub fn to_json_string(&self) -> Result<String, CueError> {
        self.validate()?;
        let ptr = unsafe { cue_value_to_json(self.0) };
        if ptr.is_null() {
            return Err(CueError::InvalidValuePointerAddress);
        }
        let c_str = unsafe { CString::from_raw(ptr) };
        Ok(c_str.to_string_lossy().into_owned())
    }

    /// Encodes the CUE value as a YAML string.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails (see [`Value::validate`]) or if
    /// the underlying C function returns a null pointer.
    pub fn to_yaml_string(&self) -> Result<String, CueError> {
        self.validate()?;
        let ptr = unsafe { cue_value_to_yaml(self.0) };
        if ptr.is_null() {
            return Err(CueError::InvalidValuePointerAddress);
        }
        let c_str = unsafe { CString::from_raw(ptr) };
        Ok(c_str.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::Value;

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

    #[test]
    fn test_to_json() {
        let value = Value::new(r#"{ name: "alice", age: 30 }"#).unwrap();
        let json = value.to_json_string().unwrap();
        let json: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(json["name"], serde_json::Value::String("alice".to_string()));
        assert_eq!(json["age"], serde_json::Value::Number(30.into()));
    }

    #[test]
    fn test_to_yaml() {
        let value = Value::new(r#"{ name: "alice", age: 30 }"#).unwrap();
        let yaml = value.to_yaml_string().unwrap();
        let yaml: serde_yml::Value = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(yaml["name"], serde_yml::Value::String("alice".to_string()));
        assert_eq!(yaml["age"], serde_yml::Value::Number(30.into()));
    }

    #[test]
    fn test_unify() {
        let value1 = Value::new("name: string, age: int ").unwrap();
        let value2 = Value::new(r#"name: "alice", age: 30"#).unwrap();
        let value3 = Value::unify(&value1, &value2);
        assert!(value3.validate().is_ok());

        let json = value3.to_json_string().unwrap();
        let json: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(json["name"], serde_json::Value::String("alice".to_string()));
        assert_eq!(json["age"], serde_json::Value::Number(30.into()));
    }
}
