//! CUE value type, wrapping the `cue_value` handle from libcue.

use core::ffi::c_char;

use crate::{Ctx, drop, error::{CueError, Error}};

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
    fn cue_dec_string(v: CueValueHandle, res: *mut *mut c_char) -> usize;
    fn cue_dec_bytes(v: CueValueHandle, res: *mut *mut core::ffi::c_void, size: *mut usize) -> usize;
    fn cue_dec_json(v: CueValueHandle, res: *mut *mut core::ffi::c_void, size: *mut usize) -> usize;
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::pedantic)]
mod tests {
    use bytes::Bytes;
    use test_case::test_case;

    use crate::{Ctx, Value, error::Error};

    // ── from_int64 ──────────────────────────────────────────────────────────

    #[test_case(0_i64; "zero")]
    #[test_case(1_i64; "one")]
    #[test_case(-1_i64; "minus_one")]
    #[test_case(i64::MAX; "max")]
    #[test_case(i64::MIN; "min")]
    fn from_int64_ok(val: i64) {
        let ctx = Ctx::new().unwrap();
        assert!(Value::from_int64(&ctx, val).is_ok());
    }

    // ── from_uint64 ─────────────────────────────────────────────────────────

    #[test_case(0_u64; "zero")]
    #[test_case(1_u64; "one")]
    #[test_case(u64::MAX; "max")]
    fn from_uint64_ok(val: u64) {
        let ctx = Ctx::new().unwrap();
        assert!(Value::from_uint64(&ctx, val).is_ok());
    }

    // ── from_bool ───────────────────────────────────────────────────────────

    #[test_case(true; "true_val")]
    #[test_case(false; "false_val")]
    fn from_bool_ok(val: bool) {
        let ctx = Ctx::new().unwrap();
        assert!(Value::from_bool(&ctx, val).is_ok());
    }

    // ── from_double ─────────────────────────────────────────────────────────

    #[test_case(0.0_f64; "zero")]
    #[test_case(1.5_f64; "positive")]
    #[test_case(-1.5_f64; "negative")]
    #[test_case(f64::MAX; "max")]
    fn from_double_ok(val: f64) {
        let ctx = Ctx::new().unwrap();
        assert!(Value::from_double(&ctx, val).is_ok());
    }

    // ── from_string ─────────────────────────────────────────────────────────

    #[test_case(""; "empty")]
    #[test_case("hello"; "ascii")]
    #[test_case("🦀 rust"; "unicode")]
    fn from_string_ok(val: &str) {
        let ctx = Ctx::new().unwrap();
        assert!(Value::from_string(&ctx, val).is_ok());
    }

    #[test]
    fn from_string_nul_byte_returns_error() {
        let ctx = Ctx::new().unwrap();
        assert!(matches!(
            Value::from_string(&ctx, "hello\0world"),
            Err(Error::StringContainsNul(_))
        ));
    }

    // ── from_bytes ──────────────────────────────────────────────────────────

    #[test_case(&b""[..]; "empty")]
    #[test_case(&b"hello"[..]; "ascii")]
    #[test_case(&[0x00_u8, 0xFF, 0x42][..]; "arbitrary")]
    fn from_bytes_ok(val: &[u8]) {
        let ctx = Ctx::new().unwrap();
        assert!(Value::from_bytes(&ctx, val).is_ok());
    }

    // ── to_string ───────────────────────────────────────────────────────────

    #[test_case(""; "empty")]
    #[test_case("hello"; "ascii")]
    #[test_case("🦀 rust"; "unicode")]
    fn to_string_roundtrip(val: &str) {
        let ctx = Ctx::new().unwrap();
        let v = Value::from_string(&ctx, val).unwrap();
        assert_eq!(v.to_string().unwrap(), val);
    }

    #[test]
    fn to_string_on_int_returns_error() {
        let ctx = Ctx::new().unwrap();
        let v = Value::from_int64(&ctx, 42).unwrap();
        assert!(matches!(v.to_string(), Err(Error::Cue(_))));
    }

    // ── to_bytes ────────────────────────────────────────────────────────────

    #[test_case(&b""[..]; "empty")]
    #[test_case(&b"hello"[..]; "ascii")]
    #[test_case(&[0x00_u8, 0xFF, 0x42][..]; "arbitrary")]
    fn to_bytes_roundtrip(val: &[u8]) {
        let ctx = Ctx::new().unwrap();
        let v = Value::from_bytes(&ctx, val).unwrap();
        assert_eq!(v.to_bytes().unwrap(), Bytes::copy_from_slice(val));
    }

    #[test]
    fn to_bytes_on_int_returns_error() {
        let ctx = Ctx::new().unwrap();
        let v = Value::from_int64(&ctx, 42).unwrap();
        assert!(matches!(v.to_bytes(), Err(Error::Cue(_))));
    }

    // ── to_json ─────────────────────────────────────────────────────────────

    #[test_case(0_i64; "zero")]
    #[test_case(42_i64; "positive")]
    #[test_case(-7_i64; "negative")]
    #[test_case(i64::MAX; "max")]
    fn to_json_from_int64(val: i64) {
        let ctx = Ctx::new().unwrap();
        let v = Value::from_int64(&ctx, val).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&v.to_json().unwrap()).unwrap();
        assert_eq!(parsed, serde_json::Value::Number(val.into()));
    }

    #[test_case(true; "true_val")]
    #[test_case(false; "false_val")]
    fn to_json_from_bool(val: bool) {
        let ctx = Ctx::new().unwrap();
        let v = Value::from_bool(&ctx, val).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&v.to_json().unwrap()).unwrap();
        assert_eq!(parsed, serde_json::Value::Bool(val));
    }

    #[test_case(""; "empty")]
    #[test_case("hello"; "ascii")]
    #[test_case("🦀 rust"; "unicode")]
    fn to_json_from_string(val: &str) {
        let ctx = Ctx::new().unwrap();
        let v = Value::from_string(&ctx, val).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&v.to_json().unwrap()).unwrap();
        assert_eq!(parsed, serde_json::Value::String(val.to_owned()));
    }

    #[test_case(0.5_f64; "half")]
    #[test_case(1.5_f64; "positive")]
    #[test_case(-1.5_f64; "negative")]
    fn to_json_from_double(val: f64) {
        let ctx = Ctx::new().unwrap();
        let v = Value::from_double(&ctx, val).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&v.to_json().unwrap()).unwrap();
        // Use bit-level equality to avoid clippy::float_cmp on exact f64 values.
        assert_eq!(parsed.as_f64().unwrap().to_bits(), val.to_bits());
    }
}
