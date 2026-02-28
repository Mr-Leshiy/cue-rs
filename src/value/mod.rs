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
/// Construct one via [`Value::compile_string`] or [`Value::compile_bytes`];
/// the underlying handle is freed automatically when this value is dropped.
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
        let err = unsafe { cue_dec_int64(self.0, &raw mut out) };
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
        let err = unsafe { cue_dec_uint64(self.0, &raw mut out) };
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
        let err = unsafe { cue_dec_bool(self.0, &raw mut out) };
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
        let err = unsafe { cue_dec_double(self.0, &raw mut out) };
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
        let err = unsafe { cue_dec_string(self.0, &raw mut ptr) };
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
        let err = unsafe { cue_dec_bytes(self.0, &raw mut ptr, &raw mut size) };
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
}
