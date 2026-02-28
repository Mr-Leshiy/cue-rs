//! CUE evaluation context, wrapping the `cue_ctx` handle from libcue.

use crate::{drop, error::Error};

/// Opaque handle to a libcue context (`cue_ctx` = `uintptr_t`).
type CueCtxHandle = usize;

unsafe extern "C" {
    /// Creates a new CUE evaluation context and returns an opaque handle.
    /// Returns 0 on failure.
    fn cue_newctx() -> CueCtxHandle;
}

/// A CUE evaluation context backed by a libcue `cue_ctx` handle.
///
/// This is the entry point for all CUE operations. Obtain one via
/// [`Ctx::new`]; the underlying context is freed automatically when this
/// value is dropped.
pub struct Ctx(CueCtxHandle);

impl Drop for Ctx {
    /// Frees the underlying libcue context via `cue_free`.
    fn drop(&mut self) {
        unsafe { drop::cue_free(self.0) }
    }
}

impl Ctx {
    /// Returns the raw libcue context handle.
    pub(crate) fn handle(&self) -> usize {
        self.0
    }

    /// Creates a new CUE evaluation context.
    ///
    /// Calls `cue_newctx` from libcue and wraps the returned handle.
    ///
    /// # Errors
    ///
    /// Returns [`CueError::ContextCreationFailed`] if `cue_newctx` returns 0,
    /// indicating that the libcue runtime could not allocate a context.
    pub fn new() -> Result<Self, Error> {
        let handle = unsafe { cue_newctx() };
        if handle == 0 {
            return Err(Error::ContextCreationFailed);
        }
        Ok(Self(handle))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::Ctx;

    #[test]
    fn test_new_succeeds() {
        assert!(Ctx::new().is_ok());
    }
}
