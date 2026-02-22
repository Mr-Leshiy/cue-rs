//! Error types returned by cue-rs operations.

use std::ffi::NulError;

use thiserror::Error;

/// Errors that can occur when working with CUE values.
#[derive(Debug, Error)]
pub enum CueError {
    /// The underlying C function returned a null pointer; the [`crate::value::Value`]
    /// handle may be dangling or was already freed.
    #[error(
        "cue_value_validate returned a null pointer; the CueValue handle may be dangling or was already freed"
    )]
    InvalidValuePointerAddress,
    /// The CUE value failed constraint validation; contains the error message.
    #[error("{0}")]
    ValidationError(String),
}

/// Errors that can occur during `Value::validate_yaml`.
#[derive(Debug, Error)]
pub enum YmlValidationError {
    /// `CueError`
    #[error(transparent)]
    CueError(#[from] CueError),
    /// `NulError`
    #[error(transparent)]
    NulError(#[from] NulError),
}
