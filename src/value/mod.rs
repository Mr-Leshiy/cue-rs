//! CUE value type, wrapping the `cue_value` handle from libcue.

#[cfg(test)]
mod tests;

use core::ffi::c_char;

use crate::{
    Ctx, drop,
    error::{CueError, Error},
};

/// Opaque handle to a libcue value (`cue_value` = `uintptr_t`).
type CueValueHandle = usize;

unsafe extern "C" {
    fn cue_validate(
        v: CueValueHandle,
        opts: *mut core::ffi::c_void,
    ) -> usize;
    fn cue_is_equal(
        a: CueValueHandle,
        b: CueValueHandle,
    ) -> bool;
    fn cue_unify(
        a: CueValueHandle,
        b: CueValueHandle,
    ) -> CueValueHandle;
    fn cue_compile_string(
        ctx: usize,
        src: *mut c_char,
        opts: *mut core::ffi::c_void,
        out: *mut CueValueHandle,
    ) -> usize;
    fn cue_compile_bytes(
        ctx: usize,
        data: *mut core::ffi::c_void,
        len: usize,
        opts: *mut core::ffi::c_void,
        out: *mut CueValueHandle,
    ) -> usize;
    fn cue_dec_json(
        v: CueValueHandle,
        res: *mut *mut core::ffi::c_void,
        size: *mut usize,
    ) -> usize;
}

/// A CUE value backed by a libcue `cue_value` handle.
///
/// Construct one via [`Value::compile_string`] or [`Value::compile_bytes`];
/// the underlying handle is freed automatically when this value is dropped.
///
/// A successfully constructed `Value` may still represent an invalid CUE
/// value (e.g. a bottom value produced by a conflicting unification).
/// Call [`Value::is_valid`] to confirm the value is error-free before using it.
#[derive(Debug)]
pub struct Value(CueValueHandle);

impl Drop for Value {
    fn drop(&mut self) {
        unsafe { drop::cue_free(self.0) }
    }
}

impl PartialEq for Value {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        unsafe { cue_is_equal(self.0, other.0) }
    }
}

impl Value {
    /// Compiles a CUE source string into a [`Value`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::StringContainsNul`] if `src` contains interior nul
    /// bytes, or [`Error::Cue`] if libcue reports a compilation error.
    pub fn compile_string(
        ctx: &Ctx,
        src: &str,
    ) -> Result<Self, Error> {
        let cstr = std::ffi::CString::new(src).map_err(Error::StringContainsNul)?;
        let mut handle: CueValueHandle = 0;
        let err = unsafe {
            cue_compile_string(
                ctx.handle(),
                cstr.as_ptr().cast_mut(),
                core::ptr::null_mut(),
                &raw mut handle,
            )
        };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        Ok(Self(handle))
    }

    /// Compiles a CUE source byte slice into a [`Value`].
    ///
    /// Unlike [`Value::compile_string`], this accepts source that may contain
    /// interior nul bytes (since it is passed by pointer and length rather than
    /// as a C string).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Cue`] if libcue reports a compilation error.
    pub fn compile_bytes(
        ctx: &Ctx,
        src: &[u8],
    ) -> Result<Self, Error> {
        let mut handle: CueValueHandle = 0;
        let err = unsafe {
            cue_compile_bytes(
                ctx.handle(),
                src.as_ptr().cast::<core::ffi::c_void>().cast_mut(),
                src.len(),
                core::ptr::null_mut(),
                &raw mut handle,
            )
        };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        Ok(Self(handle))
    }

    /// Encodes this CUE value as JSON.
    ///
    /// Calls `cue_dec_json` from libcue and copies the result into an owned
    /// [`bytes::Bytes`] buffer containing the raw JSON bytes. The C-allocated
    /// buffer is freed before returning.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Cue`] if libcue reports an error (e.g. the value
    /// cannot be represented as JSON).
    pub fn to_json_bytes(&self) -> Result<bytes::Bytes, Error> {
        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        let mut size: usize = 0;
        let err = unsafe { cue_dec_json(self.0, &raw mut ptr, &raw mut size) };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        let result = bytes::Bytes::copy_from_slice(unsafe {
            core::slice::from_raw_parts(ptr.cast::<u8>(), size)
        });
        unsafe { drop::libc_free(ptr) };
        Ok(result)
    }

    /// Unifies two CUE values, returning the meet of the two.
    ///
    /// Calls `cue_unify` from libcue.  In CUE, unification is the `&`
    /// operator: the result is the most specific value that satisfies both
    /// operands.  If the two values are incompatible the result is the bottom
    /// value (`_|_`); call [`Value::is_valid`] to check.
    #[must_use]
    pub fn unify(
        v1: &Value,
        v2: &Value,
    ) -> Self {
        let handle = unsafe { cue_unify(v1.0, v2.0) };
        Self(handle)
    }

    /// Validates this CUE value, returning an error if it is not valid.
    ///
    /// Calls `cue_validate` from libcue with no export options.  A value is
    /// valid when it contains no errors (e.g. it is not a bottom value).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Cue`] if libcue reports a validation error.
    pub fn is_valid(&self) -> Result<(), Error> {
        let err = unsafe { cue_validate(self.0, core::ptr::null_mut()) };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        Ok(())
    }
}
