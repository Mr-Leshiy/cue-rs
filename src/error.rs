use thiserror::Error;

#[derive(Debug, Error)]
pub enum CueError {
    #[error("cue_value_validate returned a null pointer; the CueValue handle may be dangling or was already freed")]
    InvalidValuePointerAddress,
    #[error("{0}")]
    ValidationError(String),
}
