#![allow(clippy::unwrap_used, clippy::pedantic)]

use bytes::Bytes;
use test_case::test_case;

use crate::{Ctx, Value, error::Error};

// ── int64 ──────────────────────────────────────────────────────────

#[test_case(0_i64; "zero")]
#[test_case(1_i64; "one")]
#[test_case(-1_i64; "minus_one")]
#[test_case(i64::MAX; "max")]
// TODO: internal libcue bug of processing `i64::MIN` `-9223372036854775808`  value
#[test_case(i64::MIN + 1; "min")]
fn from_int64_ok(val: i64) {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_int64(&ctx, val).unwrap();
    assert_eq!(v.to_int64().unwrap(), val);
}

#[test]
fn to_int64_on_string_returns_error() {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_string(&ctx, "hello").unwrap();
    assert!(matches!(v.to_int64(), Err(Error::Cue(_))));
}

// ── uint64 ─────────────────────────────────────────────────────────

#[test_case(0_u64; "zero")]
#[test_case(1_u64; "one")]
#[test_case(u64::MAX; "max")]
fn from_uint64_ok(val: u64) {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_uint64(&ctx, val).unwrap();
    assert_eq!(v.to_uint64().unwrap(), val);
}

#[test]
fn to_uint64_on_string_returns_error() {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_string(&ctx, "hello").unwrap();
    assert!(matches!(v.to_uint64(), Err(Error::Cue(_))));
}

// ── bool ───────────────────────────────────────────────────────────

#[test_case(true; "true_val")]
#[test_case(false; "false_val")]
fn from_bool_ok(val: bool) {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_bool(&ctx, val).unwrap();
    assert_eq!(v.to_bool().unwrap(), val);
}

#[test]
fn to_bool_on_int_returns_error() {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_int64(&ctx, 1).unwrap();
    assert!(matches!(v.to_bool(), Err(Error::Cue(_))));
}

// ── double ─────────────────────────────────────────────────────────

#[test_case(0.0_f64; "zero")]
#[test_case(1.5_f64; "positive")]
#[test_case(-1.5_f64; "negative")]
#[test_case(f64::MAX; "max")]
fn from_double_ok(val: f64) {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_double(&ctx, val).unwrap();
    assert_eq!(v.to_double().unwrap(), val);
}

#[test]
fn to_double_on_string_returns_error() {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_string(&ctx, "1.5").unwrap();
    assert!(matches!(v.to_double(), Err(Error::Cue(_))));
}

// ── string ─────────────────────────────────────────────────────────

#[test_case(""; "empty")]
#[test_case("hello"; "ascii")]
#[test_case("🦀 rust"; "unicode")]
fn from_string_ok(val: &str) {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_string(&ctx, val).unwrap();
    assert_eq!(v.to_string().unwrap(), val);
}

#[test]
fn from_string_nul_byte_returns_error() {
    let ctx = Ctx::new().unwrap();
    assert!(matches!(
        Value::from_string(&ctx, "hello\0world"),
        Err(Error::StringContainsNul(_))
    ));
}

#[test]
fn to_string_on_int_returns_error() {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_int64(&ctx, 42).unwrap();
    assert!(matches!(v.to_string(), Err(Error::Cue(_))));
}

// ── bytes ──────────────────────────────────────────────────────────

#[test_case(&b""[..]; "empty")]
#[test_case(&b"hello"[..]; "ascii")]
#[test_case(&[0x00_u8, 0xFF, 0x42][..]; "arbitrary")]
fn from_bytes_ok(val: &[u8]) {
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
    let parsed: serde_json::Value = serde_json::from_slice(&v.to_json().unwrap()).unwrap();
    assert_eq!(parsed, serde_json::Value::Number(val.into()));
}

#[test_case(true; "true_val")]
#[test_case(false; "false_val")]
fn to_json_from_bool(val: bool) {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_bool(&ctx, val).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&v.to_json().unwrap()).unwrap();
    assert_eq!(parsed, serde_json::Value::Bool(val));
}

#[test_case(""; "empty")]
#[test_case("hello"; "ascii")]
#[test_case("🦀 rust"; "unicode")]
fn to_json_from_string(val: &str) {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_string(&ctx, val).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&v.to_json().unwrap()).unwrap();
    assert_eq!(parsed, serde_json::Value::String(val.to_owned()));
}

#[test_case(0.5_f64; "half")]
#[test_case(1.5_f64; "positive")]
#[test_case(-1.5_f64; "negative")]
fn to_json_from_double(val: f64) {
    let ctx = Ctx::new().unwrap();
    let v = Value::from_double(&ctx, val).unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&v.to_json().unwrap()).unwrap();
    // Use bit-level equality to avoid clippy::float_cmp on exact f64 values.
    assert_eq!(parsed.as_f64().unwrap().to_bits(), val.to_bits());
}

// ── PartialEq ───────────────────────────────────────────────────────

// int64
#[test_case(|ctx: &Ctx| Value::from_int64(ctx, 0),|ctx: &Ctx| Value::from_int64(ctx, 0) => true; "0_u64 == 0_u64")]
#[test_case(|ctx: &Ctx| Value::from_int64(ctx, 0),|ctx: &Ctx| Value::from_int64(ctx, 1) => false; "0_u64 != 1_u64")]
#[test_case(|ctx: &Ctx| Value::from_int64(ctx, -1), |ctx: &Ctx| Value::from_int64(ctx, -1) => true; "int64_neg_eq")]
#[test_case(|ctx: &Ctx| Value::from_int64(ctx, i64::MAX), |ctx: &Ctx| Value::from_int64(ctx, i64::MAX) => true; "int64_max_eq")]
#[test_case(|ctx: &Ctx| Value::from_int64(ctx, i64::MAX), |ctx: &Ctx| Value::from_int64(ctx, i64::MIN + 1) => false; "int64_max_ne_min")]
// uint64
#[test_case(|ctx: &Ctx| Value::from_uint64(ctx, 0), |ctx: &Ctx| Value::from_uint64(ctx, 0) => true; "uint64_zero_eq")]
#[test_case(|ctx: &Ctx| Value::from_uint64(ctx, 1), |ctx: &Ctx| Value::from_uint64(ctx, 1) => true; "uint64_one_eq")]
#[test_case(|ctx: &Ctx| Value::from_uint64(ctx, u64::MAX), |ctx: &Ctx| Value::from_uint64(ctx, u64::MAX) => true; "uint64_max_eq")]
#[test_case(|ctx: &Ctx| Value::from_uint64(ctx, 0), |ctx: &Ctx| Value::from_uint64(ctx, 1) => false; "uint64_zero_ne_one")]
// bool
#[test_case(|ctx: &Ctx| Value::from_bool(ctx, true), |ctx: &Ctx| Value::from_bool(ctx, true) => true; "bool_true_eq")]
#[test_case(|ctx: &Ctx| Value::from_bool(ctx, false), |ctx: &Ctx| Value::from_bool(ctx, false) => true; "bool_false_eq")]
#[test_case(|ctx: &Ctx| Value::from_bool(ctx, true), |ctx: &Ctx| Value::from_bool(ctx, false) => false; "bool_true_ne_false")]
// double
#[test_case(|ctx: &Ctx| Value::from_double(ctx, 0.0), |ctx: &Ctx| Value::from_double(ctx, 0.0) => true; "double_zero_eq")]
#[test_case(|ctx: &Ctx| Value::from_double(ctx, 1.5), |ctx: &Ctx| Value::from_double(ctx, 1.5) => true; "double_pos_eq")]
#[test_case(|ctx: &Ctx| Value::from_double(ctx, -1.5), |ctx: &Ctx| Value::from_double(ctx, -1.5) => true; "double_neg_eq")]
#[test_case(|ctx: &Ctx| Value::from_double(ctx, 1.5), |ctx: &Ctx| Value::from_double(ctx, 2.5) => false; "double_ne")]
// string
#[test_case(|ctx: &Ctx| Value::from_string(ctx, ""), |ctx: &Ctx| Value::from_string(ctx, "") => true; "string_empty_eq")]
#[test_case(|ctx: &Ctx| Value::from_string(ctx, "hello"), |ctx: &Ctx| Value::from_string(ctx, "hello") => true; "string_ascii_eq")]
#[test_case(|ctx: &Ctx| Value::from_string(ctx, "🦀"), |ctx: &Ctx| Value::from_string(ctx, "🦀") => true; "string_unicode_eq")]
#[test_case(|ctx: &Ctx| Value::from_string(ctx, "hello"), |ctx: &Ctx| Value::from_string(ctx, "world") => false; "string_ne")]
// bytes
#[test_case(|ctx: &Ctx| Value::from_bytes(ctx, b""), |ctx: &Ctx| Value::from_bytes(ctx, b"") => true; "bytes_empty_eq")]
#[test_case(|ctx: &Ctx| Value::from_bytes(ctx, b"hello"), |ctx: &Ctx| Value::from_bytes(ctx, b"hello") => true; "bytes_ascii_eq")]
#[test_case(|ctx: &Ctx| Value::from_bytes(ctx, &[0x00, 0xFF]), |ctx: &Ctx| Value::from_bytes(ctx, &[0x00, 0xFF]) => true; "bytes_arbitrary_eq")]
#[test_case(|ctx: &Ctx| Value::from_bytes(ctx, b"foo"), |ctx: &Ctx| Value::from_bytes(ctx, b"bar") => false; "bytes_ne")]
fn value_equal_test(
    a: impl FnOnce(&Ctx) -> Result<Value, Error>,
    b: impl FnOnce(&Ctx) -> Result<Value, Error>,
) -> bool {
    let ctx = Ctx::new().unwrap();
    let a = a(&ctx).unwrap();
    let b = b(&ctx).unwrap();
    a == b
}
