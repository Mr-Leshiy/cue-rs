#![allow(clippy::unwrap_used, clippy::pedantic)]

use serde_json::json;
use test_case::test_case;

use crate::{Ctx, Value, error::Error};

// ── int64 ──────────────────────────────────────────────────────────

#[test_case(
    0.to_string()
    => json!({});
    "int zero"
)]
#[test_case(
    1.to_string()
    => json!({});
    "int one"
)]
#[test_case(
    (-1).to_string()
    => json!({});
    "int minus one"
)]
#[test_case(
    i32::MAX.to_string()
    => json!({});
    "int max"
)]
#[test_case(
    i32::MIN.to_string()
    => json!({});
    "int min"
)]
#[test_case(
    true.to_string()=>
    json!({});
    "true boolean"
)]
#[test_case(
    false.to_string()
    => json!({});
    "false boolean"
)]
#[test_case(
    0.0.to_string()
    => json!({});
    "double zero"
)]
#[test_case(
    1.5.to_string()
    => json!({});
    "double positive"
)]
#[test_case(
    (-1.5).to_string()
    => json!({});
    "double negative"
)]
#[test_case(
    f32::MAX.to_string()
    => json!({});
    "double max"
)]
#[test_case(
    f32::MIN.to_string()
    => json!({});
    "double min"
)]
#[test_case(
    format!(r#""""#)
    => json!({});
    "empty"
)]
#[test_case(
    format!(r#""hello""#)
    => json!({});
    "ascii"
)]
#[test_case(
    format!(r#""🦀 rust""#)
    => json!({});
    "unicode"
)]
fn value_test(val: String) -> serde_json::Value {
    let ctx = Ctx::new().unwrap();
    let v = Value::compile_string(&ctx, &val).unwrap();
    let v_from_bytes = Value::compile_bytes(&ctx, val.as_bytes()).unwrap();
    assert_eq!(v, v_from_bytes);
    let v_json = serde_json::from_slice::<serde_json::Value>(&v.to_json_bytes().unwrap()).unwrap();
    let v_from_bytes_json = serde_json::from_slice::<serde_json::Value>(&v.to_json_bytes().unwrap()).unwrap();
    assert_eq!(v_json, v_from_bytes_json);
    v_json
}

// ── PartialEq ───────────────────────────────────────────────────────

// int64
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "0"), |ctx: &Ctx| Value::compile_string(ctx, "0") => true; "0_u64 == 0_u64")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "0"), |ctx: &Ctx| Value::compile_string(ctx, "1") => false; "0_u64 != 1_u64")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "-1"), |ctx: &Ctx| Value::compile_string(ctx, "-1") => true; "int64_neg_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "9223372036854775807"), |ctx: &Ctx| Value::compile_string(ctx, "9223372036854775807") => true; "int64_max_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "9223372036854775807"), |ctx: &Ctx| Value::compile_string(ctx, "-9223372036854775807") => false; "int64_max_ne_min")]
// uint64
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "0"), |ctx: &Ctx| Value::compile_string(ctx, "0") => true; "uint64_zero_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "1"), |ctx: &Ctx| Value::compile_string(ctx, "1") => true; "uint64_one_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "18446744073709551615"), |ctx: &Ctx| Value::compile_string(ctx, "18446744073709551615") => true; "uint64_max_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "0"), |ctx: &Ctx| Value::compile_string(ctx, "1") => false; "uint64_zero_ne_one")]
// bool
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "true"), |ctx: &Ctx| Value::compile_string(ctx, "true") => true; "bool_true_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "false"), |ctx: &Ctx| Value::compile_string(ctx, "false") => true; "bool_false_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "true"), |ctx: &Ctx| Value::compile_string(ctx, "false") => false; "bool_true_ne_false")]
// double
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "0.0"), |ctx: &Ctx| Value::compile_string(ctx, "0.0") => true; "double_zero_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "1.5"), |ctx: &Ctx| Value::compile_string(ctx, "1.5") => true; "double_pos_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "-1.5"), |ctx: &Ctx| Value::compile_string(ctx, "-1.5") => true; "double_neg_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "1.5"), |ctx: &Ctx| Value::compile_string(ctx, "2.5") => false; "double_ne")]
// string
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, r#""""#), |ctx: &Ctx| Value::compile_string(ctx, r#""""#) => true; "string_empty_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, r#""hello""#), |ctx: &Ctx| Value::compile_string(ctx, r#""hello""#) => true; "string_ascii_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, r#""🦀""#), |ctx: &Ctx| Value::compile_string(ctx, r#""🦀""#) => true; "string_unicode_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, r#""hello""#), |ctx: &Ctx| Value::compile_string(ctx, r#""world""#) => false; "string_ne")]
// bytes
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "''"), |ctx: &Ctx| Value::compile_string(ctx, "''") => true; "bytes_empty_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "'hello'"), |ctx: &Ctx| Value::compile_string(ctx, "'hello'") => true; "bytes_ascii_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, r"'\x00\xff'"), |ctx: &Ctx| Value::compile_string(ctx, r"'\x00\xff'") => true; "bytes_arbitrary_eq")]
#[test_case(|ctx: &Ctx| Value::compile_string(ctx, "'foo'"), |ctx: &Ctx| Value::compile_string(ctx, "'bar'") => false; "bytes_ne")]
fn value_equal_test(
    a: impl FnOnce(&Ctx) -> Result<Value, Error>,
    b: impl FnOnce(&Ctx) -> Result<Value, Error>,
) -> bool {
    let ctx = Ctx::new().unwrap();
    let a = a(&ctx).unwrap();
    let b = b(&ctx).unwrap();
    a == b
}
