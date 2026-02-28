//! Error types returned by cue-rs operations.

use core::ffi::c_char;

use thiserror::Error;

/// Opaque handle type matching `typedef uintptr_t cue_error` from libcue.
type CueErrorHandle = usize;

unsafe extern "C" {
    fn cue_error_string(err: CueErrorHandle) -> *mut c_char;
}

/// A libcue error handle (`cue_error`).
#[derive(Debug)]
pub struct CueError(pub(crate) CueErrorHandle);

impl std::fmt::Display for CueError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let ptr = unsafe { cue_error_string(self.0) };
        if ptr.is_null() {
            return f.write_str("<unknown cue error>");
        }
        let s = unsafe { std::ffi::CStr::from_ptr(ptr) }.to_string_lossy();
        let result = f.write_str(&s);
        unsafe { crate::drop::libc_free(ptr.cast()) };
        result
    }
}

/// Errors that can occur when working with CUE values.
#[derive(Debug, Error)]
pub enum Error {
    /// `cue_newctx` returned 0; the libcue runtime could not allocate a
    /// context.
    #[error("cue_newctx returned 0; the libcue runtime could not allocate a context")]
    ContextCreationFailed,

    /// The string passed to `cue_compile_string` contains an interior nul byte.
    #[error("string contains an interior nul byte: {0}")]
    StringContainsNul(std::ffi::NulError),

    /// A libcue operation returned a `cue_error` handle.
    #[error("{0}")]
    Cue(CueError),

    /// A string decoded from libcue was not valid UTF-8.
    #[error("decoded string is not valid UTF-8: {0}")]
    InvalidUtf8(std::str::Utf8Error),
}
