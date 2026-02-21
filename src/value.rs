use std::ffi::{CString, NulError};
use std::os::raw::c_char;

use crate::error::CueError;

/// Opaque handle to a compiled CUE value managed by the Go runtime.
/// Obtain one with [`go_cue_value_new`] and release it with [`go_cue_value_free`].
type CueValueAddr = usize;

unsafe extern "C" {
    fn cue_value_new(input: *const c_char) -> CueValueAddr;
    fn cue_value_free(addr: CueValueAddr);
    fn cue_value_unify(addr1: CueValueAddr, addr2: CueValueAddr) -> CueValueAddr;
    /// Returns a malloc-allocated C string; caller must free it.
    fn cue_value_validate(addr: CueValueAddr) -> *mut c_char;
    /// Returns a malloc-allocated JSON string, or NULL on error; caller must free it.
    fn cue_value_to_json(addr: CueValueAddr) -> *mut c_char;
    /// Returns a malloc-allocated YAML string, or NULL on error; caller must free it.
    fn cue_value_to_yaml(addr: CueValueAddr) -> *mut c_char;
}

pub struct Value(CueValueAddr);

impl Drop for Value {
    /// Releases the Go-side storage associated with `handle`.
    fn drop(&mut self) {
        unsafe { cue_value_free(self.0) }
    }
}

impl Value {
    /// Compiles `input` as a CUE expression and returns an [`Value`].
    ///
    /// # Errors:
    /// This function will return an error if the supplied string contains an
    /// internal 0 byte. The [`NulError`] returned will contain the bytes as well as
    /// the position of the nul byte.
    pub fn new(s: &str) -> Result<Self, NulError> {
        let c_str = CString::new(s)?;
        Ok(Self(unsafe { cue_value_new(c_str.as_ptr()) }))
    }

    /// <https://pkg.go.dev/cuelang.org/go/cue#Value.Unify>
    pub fn unify(val1: &Value, val2: &Value) -> Result<Value, CueError> {
        Ok(Self( unsafe { cue_value_unify(val1.0, val2.0) }))
    }

    /// Validates the CUE value and returns underlying error message.
    /// <https://pkg.go.dev/cuelang.org/go/cue#Value.Validate>
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
        let value1 = Value::new(r#"name: string, age: int "#).unwrap();
        let value2 = Value::new(r#"name: "alice", age: 30"#).unwrap();
        let value3 = Value::unify(&value1, &value2).unwrap();
        assert!(value3.validate().is_ok());

        let json = value3.to_json_string().unwrap();
        let json: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(json["name"], serde_json::Value::String("alice".to_string()));
        assert_eq!(json["age"], serde_json::Value::Number(30.into()));
    }
}
