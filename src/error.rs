//! Error types returned by cue-rs operations.

use thiserror::Error;

/// Errors that can occur when working with CUE values.
#[derive(Debug, Error)]
pub enum CueError {
    /// `cue_newctx` returned 0; the libcue runtime could not allocate a
    /// context.
    #[error("cue_newctx returned 0; the libcue runtime could not allocate a context")]
    ContextCreationFailed,

    /// A `cue_from_*` function returned 0; libcue could not create the value.
    #[error("cue_from_* returned 0; libcue could not create the value")]
    ValueCreationFailed,

    /// The string passed to `cue_from_string` contains an interior nul byte.
    #[error("string contains an interior nul byte: {0}")]
    StringContainsNul(std::ffi::NulError),
}
