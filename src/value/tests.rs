#![allow(clippy::unwrap_used, clippy::pedantic)]

use serde_json::json;
use test_case::test_case;

use crate::{Ctx, Value};

// ── int64 ──────────────────────────────────────────────────────────

#[test_case(
    0.to_string()
    => json!(0);
    "int zero"
)]
#[test_case(
    1.to_string()
    => json!(1);
    "int one"
)]
#[test_case(
    (-1).to_string()
    => json!(-1);
    "int minus one"
)]
#[test_case(
    i32::MAX.to_string()
    => json!(2147483647);
    "int max"
)]
#[test_case(
    i32::MIN.to_string()
    => json!(-2147483648);
    "int min"
)]
#[test_case(
    true.to_string()=>
    json!(true);
    "true boolean"
)]
#[test_case(
    false.to_string()
    => json!(false);
    "false boolean"
)]
#[test_case(
    0.0.to_string()
    => json!(0);
    "double zero"
)]
#[test_case(
    1.5.to_string()
    => json!(1.5);
    "double positive"
)]
#[test_case(
    (-1.5).to_string()
    => json!(-1.5);
    "double negative"
)]
#[test_case(
    f32::MAX.to_string()
    => json!(3.4028235e38_f64);
    "double max"
)]
#[test_case(
    f32::MIN.to_string()
    => json!(-3.4028235e38_f64);
    "double min"
)]
#[test_case(
    format!(r#""""#)
    => json!("");
    "empty"
)]
#[test_case(
    format!(r#""hello""#)
    => json!("hello");
    "ascii"
)]
#[test_case(
    format!(r#""🦀 rust""#)
    => json!("🦀 rust");
    "unicode"
)]
fn value_test(val: String) -> serde_json::Value {
    let ctx = Ctx::new().unwrap();
    let v = Value::compile_string(&ctx, &val).unwrap();
    let v_from_bytes = Value::compile_bytes(&ctx, val.as_bytes()).unwrap();
    assert_eq!(v, v_from_bytes);
    let v_json = serde_json::from_slice::<serde_json::Value>(&v.to_json_bytes().unwrap()).unwrap();
    let v_from_bytes_json =
        serde_json::from_slice::<serde_json::Value>(&v.to_json_bytes().unwrap()).unwrap();
    assert_eq!(v_json, v_from_bytes_json);
    v_json
}

// ── unify ─────────────────────────────────────────────────────────────

#[test_case("42",         "42"     => json!(42);    "identical ints")]
#[test_case("true",       "bool"   => json!(true);  "bool value meets bool type")]
#[test_case(r#""hello""#, "string" => json!("hello"); "string value meets string type")]
#[test_case("1.5",        "number" => json!(1.5);   "float value meets number type")]
#[test_case(">0",         "42"     => json!(42);    "constraint meets concrete int")]
fn value_unify_test(
    a: &str,
    b: &str,
) -> serde_json::Value {
    let ctx = Ctx::new().unwrap();
    let va = Value::compile_string(&ctx, a).unwrap();
    let vb = Value::compile_string(&ctx, b).unwrap();
    let v = Value::unify(&va, &vb);
    serde_json::from_slice::<serde_json::Value>(&v.to_json_bytes().unwrap()).unwrap()
}

#[test_case("1",      "2"      ; "conflicting ints produce bottom")]
#[test_case(r#""a""#, r#""b""# ; "conflicting strings produce bottom")]
fn value_unify_bottom_test(
    a: &str,
    b: &str,
) {
    let ctx = Ctx::new().unwrap();
    let va = Value::compile_string(&ctx, a).unwrap();
    let vb = Value::compile_string(&ctx, b).unwrap();
    assert!(Value::unify(&va, &vb).is_valid().is_err());
}

// ── is_valid ─────────────────────────────────────────────────────────

#[test_case("42"        => true;  "int is valid")]
#[test_case("true"      => true;  "bool is valid")]
#[test_case("1.5"       => true;  "float is valid")]
#[test_case(r#""hello""# => true;  "string is valid")]
#[test_case("_|_"       => false; "bottom is invalid")]
#[test_case("1 & 2"     => false; "conflicting unification is invalid")]
fn value_valid_test(src: &str) -> bool {
    let ctx = Ctx::new().unwrap();
    match Value::compile_string(&ctx, src) {
        Err(_) => false,
        Ok(v) => v.is_valid().is_ok(),
    }
}
