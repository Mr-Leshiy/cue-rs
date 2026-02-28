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
    fn cue_is_equal(
        a: CueValueHandle,
        b: CueValueHandle,
    ) -> bool;
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
    fn cue_dec_int64(
        v: CueValueHandle,
        res: *mut i64,
    ) -> usize;
    fn cue_dec_uint64(
        v: CueValueHandle,
        res: *mut u64,
    ) -> usize;
    fn cue_dec_bool(
        v: CueValueHandle,
        res: *mut bool,
    ) -> usize;
    fn cue_dec_double(
        v: CueValueHandle,
        res: *mut f64,
    ) -> usize;
    fn cue_dec_string(
        v: CueValueHandle,
        res: *mut *mut c_char,
    ) -> usize;
    fn cue_dec_bytes(
        v: CueValueHandle,
        res: *mut *mut core::ffi::c_void,
        size: *mut usize,
    ) -> usize;
    fn cue_dec_json(
        v: CueValueHandle,
        res: *mut *mut core::ffi::c_void,
        size: *mut usize,
    ) -> usize;
}

/// A CUE value backed by a libcue `cue_value` handle.
///
/// Construct one via the `Value::from_*` family of methods; the underlying
/// handle is freed automatically when this value is dropped.
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
    /// Creates a CUE integer value from an [`i64`].
    ///
    /// # Errors
    ///
    /// Returns [`CueError::ValueCreationFailed`] if libcue returns 0.
    pub fn from_int64(
        ctx: &Ctx,
        val: i64,
    ) -> Result<Self, Error> {
        let handle = unsafe { cue_from_int64(ctx.handle(), val) };
        if handle == 0 {
            return Err(Error::ValueCreationFailed);
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
    ) -> Result<Self, Error> {
        let handle = unsafe { cue_from_uint64(ctx.handle(), val) };
        if handle == 0 {
            return Err(Error::ValueCreationFailed);
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
    ) -> Result<Self, Error> {
        let handle = unsafe { cue_from_bool(ctx.handle(), val) };
        if handle == 0 {
            return Err(Error::ValueCreationFailed);
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
    ) -> Result<Self, Error> {
        let handle = unsafe { cue_from_double(ctx.handle(), val) };
        if handle == 0 {
            return Err(Error::ValueCreationFailed);
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
    ) -> Result<Self, Error> {
        let cstr = std::ffi::CString::new(val).map_err(|e| Error::StringContainsNul(e))?;
        let handle = unsafe { cue_from_string(ctx.handle(), cstr.as_ptr().cast_mut()) };
        if handle == 0 {
            return Err(Error::ValueCreationFailed);
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
    ) -> Result<Self, Error> {
        let handle = unsafe {
            cue_from_bytes(
                ctx.handle(),
                val.as_ptr().cast::<core::ffi::c_void>().cast_mut(),
                val.len(),
            )
        };
        if handle == 0 {
            return Err(Error::ValueCreationFailed);
        }
        Ok(Self(handle))
    }

    /// Decodes this CUE value as an [`i64`].
    ///
    /// Calls `cue_dec_int64` from libcue.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Cue`] if libcue reports an error (e.g. the value is
    /// not a CUE integer, or it does not fit in an [`i64`]).
    pub fn to_int64(&self) -> Result<i64, Error> {
        let mut out: i64 = 0;
        let err = unsafe { cue_dec_int64(self.0, &mut out) };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        Ok(out)
    }

    /// Decodes this CUE value as a [`u64`].
    ///
    /// Calls `cue_dec_uint64` from libcue.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Cue`] if libcue reports an error (e.g. the value is
    /// not a CUE integer, or it does not fit in a [`u64`]).
    pub fn to_uint64(&self) -> Result<u64, Error> {
        let mut out: u64 = 0;
        let err = unsafe { cue_dec_uint64(self.0, &mut out) };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        Ok(out)
    }

    /// Decodes this CUE value as a [`bool`].
    ///
    /// Calls `cue_dec_bool` from libcue.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Cue`] if libcue reports an error (e.g. the value is
    /// not a CUE boolean).
    pub fn to_bool(&self) -> Result<bool, Error> {
        let mut out: bool = false;
        let err = unsafe { cue_dec_bool(self.0, &mut out) };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        Ok(out)
    }

    /// Decodes this CUE value as an [`f64`].
    ///
    /// Calls `cue_dec_double` from libcue.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Cue`] if libcue reports an error (e.g. the value is
    /// not a CUE number).
    pub fn to_double(&self) -> Result<f64, Error> {
        let mut out: f64 = 0.0;
        let err = unsafe { cue_dec_double(self.0, &mut out) };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        Ok(out)
    }

    /// Decodes this CUE value as a UTF-8 string.
    ///
    /// Calls `cue_dec_string` from libcue and copies the result into an owned
    /// [`String`]. The C-allocated buffer is freed before returning.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Cue`] if libcue reports an error (e.g. the value is not
    /// a CUE string), or [`Error::InvalidUtf8`] if the bytes returned by libcue
    /// are not valid UTF-8.
    pub fn to_string(&self) -> Result<String, Error> {
        let mut ptr: *mut c_char = core::ptr::null_mut();
        let err = unsafe { cue_dec_string(self.0, &mut ptr) };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        let result = unsafe { core::ffi::CStr::from_ptr(ptr) }
            .to_str()
            .map(str::to_owned)
            .map_err(Error::InvalidUtf8);
        unsafe { drop::libc_free(ptr.cast()) };
        result
    }

    /// Decodes this CUE value as a byte slice.
    ///
    /// Calls `cue_dec_bytes` from libcue and copies the result into an owned
    /// [`bytes::Bytes`] buffer. The C-allocated buffer is freed before
    /// returning.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Cue`] if libcue reports an error (e.g. the value is not
    /// a CUE bytes literal).
    pub fn to_bytes(&self) -> Result<bytes::Bytes, Error> {
        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        let mut size: usize = 0;
        let err = unsafe { cue_dec_bytes(self.0, &mut ptr, &mut size) };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        let result = bytes::Bytes::copy_from_slice(unsafe {
            core::slice::from_raw_parts(ptr.cast::<u8>(), size)
        });
        unsafe { drop::libc_free(ptr) };
        Ok(result)
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
    pub fn to_json(&self) -> Result<bytes::Bytes, Error> {
        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        let mut size: usize = 0;
        let err = unsafe { cue_dec_json(self.0, &mut ptr, &mut size) };
        if err != 0 {
            return Err(Error::Cue(CueError(err)));
        }
        let result = bytes::Bytes::copy_from_slice(unsafe {
            core::slice::from_raw_parts(ptr.cast::<u8>(), size)
        });
        unsafe { drop::libc_free(ptr) };
        Ok(result)
    }
}
