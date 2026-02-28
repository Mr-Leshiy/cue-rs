//! Error types returned by cue-rs operations.

use thiserror::Error;

/// Errors that can occur when working with CUE values.
#[derive(Debug, Error)]
pub enum CueError {
    /// `cue_newctx` returned 0; the libcue runtime could not allocate a
    /// context.
    #[error("cue_newctx returned 0; the libcue runtime could not allocate a context")]
    ContextCreationFailed,
}