//! CUE value type, wrapping the `cue_value` handle from libcue.

use core::ffi::c_char;

use crate::{Ctx, drop, error::CueError};

/// Opaque handle to a libcue value (`cue_value` = `uintptr_t`).
type CueValueHandle = usize;

unsafe extern "C" {
    fn cue_from_int64(
        ctx: usize,
        val: i64,
    ) -> CueValueHandle;
    fn cue_from_uint64(
        ctx: usize,
        val: u64,
    ) -> CueValueHandle;
    fn cue_from_bool(
        ctx: usize,
        val: bool,
    ) -> CueValueHandle;
    fn cue_from_double(
        ctx: usize,
        val: f64,
    ) -> CueValueHandle;
    fn cue_from_string(
        ctx: usize,
        val: *mut c_char,
    ) -> CueValueHandle;
    fn cue_from_bytes(
        ctx: usize,
        data: *mut core::ffi::c_void,
        len: usize,
    ) -> CueValueHandle;
}

/// A CUE value backed by a libcue `cue_value` handle.
///
/// Construct one via the `Value::from_*` family of methods; the underlying
/// handle is freed automatically when this value is dropped.
pub struct Value(CueValueHandle);

impl Drop for Value {
    fn drop(&mut self) {
        unsafe { drop::cue_free(self.0) }
    }
}

impl Value {
    /// Creates a CUE integer value from an [`i64`].
    ///
    /// # Errors
    ///
    /// Returns [`CueError::ValueCreationFailed`] if libcue returns 0.
    pub fn from_int64(
        ctx: &Ctx,
        val: i64,
    ) -> Result<Self, CueError> {
        let handle = unsafe { cue_from_int64(ctx.handle(), val) };
        if handle == 0 {
            return Err(CueError::ValueCreationFailed);
        }
        Ok(Self(handle))
    }

    /// Creates a CUE integer value from a [`u64`].
    ///
    /// # Errors
    ///
    /// Returns [`CueError::ValueCreationFailed`] if libcue returns 0.
    pub fn from_uint64(
        ctx: &Ctx,
        val: u64,
    ) -> Result<Self, CueError> {
        let handle = unsafe { cue_from_uint64(ctx.handle(), val) };
        if handle == 0 {
            return Err(CueError::ValueCreationFailed);
        }
        Ok(Self(handle))
    }

    /// Creates a CUE boolean value from a [`bool`].
    ///
    /// # Errors
    ///
    /// Returns [`CueError::ValueCreationFailed`] if libcue returns 0.
    pub fn from_bool(
        ctx: &Ctx,
        val: bool,
    ) -> Result<Self, CueError> {
        let handle = unsafe { cue_from_bool(ctx.handle(), val) };
        if handle == 0 {
            return Err(CueError::ValueCreationFailed);
        }
        Ok(Self(handle))
    }

    /// Creates a CUE float value from an [`f64`].
    ///
    /// # Errors
    ///
    /// Returns [`CueError::ValueCreationFailed`] if libcue returns 0.
    pub fn from_double(
        ctx: &Ctx,
        val: f64,
    ) -> Result<Self, CueError> {
        let handle = unsafe { cue_from_double(ctx.handle(), val) };
        if handle == 0 {
            return Err(CueError::ValueCreationFailed);
        }
        Ok(Self(handle))
    }

    /// Creates a CUE string value from a Rust `&str`.
    ///
    /// # Errors
    ///
    /// Returns [`CueError::StringContainsNul`] if `val` contains interior nul
    /// bytes, or [`CueError::ValueCreationFailed`] if libcue returns 0.
    pub fn from_string(
        ctx: &Ctx,
        val: &str,
    ) -> Result<Self, CueError> {
        let cstr = std::ffi::CString::new(val).map_err(|e| CueError::StringContainsNul(e))?;
        let handle = unsafe { cue_from_string(ctx.handle(), cstr.as_ptr().cast_mut()) };
        if handle == 0 {
            return Err(CueError::ValueCreationFailed);
        }
        Ok(Self(handle))
    }

    /// Creates a CUE bytes value from a Rust `&[u8]`.
    ///
    /// # Errors
    ///
    /// Returns [`CueError::ValueCreationFailed`] if libcue returns 0.
    pub fn from_bytes(
        ctx: &Ctx,
        val: &[u8],
    ) -> Result<Self, CueError> {
        let handle = unsafe {
            cue_from_bytes(
                ctx.handle(),
                val.as_ptr().cast::<core::ffi::c_void>().cast_mut(),
                val.len(),
            )
        };
        if handle == 0 {
            return Err(CueError::ValueCreationFailed);
        }
        Ok(Self(handle))
    }
}
