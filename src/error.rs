//! Error types returned by cue-rs operations.

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
