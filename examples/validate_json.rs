//! Validate a [`serde_json::Value`] against a CUE schema.
//!
//! The approach:
//! 1. Compile the CUE schema string into a [`cue_rs::Value`].
//! 2. Serialize the JSON data to bytes and compile it into a second [`cue_rs::Value`].
//! 3. Unify the schema and the data — in CUE, unification is the `&` operator.
//! 4. Call [`cue_rs::Value::is_valid`] on the result; a bottom value (`_|_`) means
//!    the data does not conform to the schema.

use cue_rs::{Ctx, Value, error::Error};

/// Validates `data` against `schema` by unifying them and checking [`Value::is_valid`].
fn validate(
    ctx: &Ctx,
    schema: &Value,
    data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_bytes = serde_json::to_vec(data)?;
    let data_val = Value::compile_bytes(ctx, &json_bytes)?;
    Value::unify(schema, &data_val).is_valid()?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let ctx = Ctx::new()?;

    // CUE schema: an object with a `string` name and a non-negative `int` age.
    let schema = Value::compile_string(&ctx, r#"{ name: string, age: int & >=0 }"#)?;

    // ✓ Valid: both fields satisfy the schema.
    let valid = serde_json::json!({ "name": "alice", "age": 30 });
    match validate(&ctx, &schema, &valid) {
        Ok(()) => println!("valid:    {valid}"),
        Err(e) => println!("unexpected failure: {e}"),
    }

    // ✗ Invalid: `age` is negative, violating `>=0`.
    let invalid_age = serde_json::json!({ "name": "bob", "age": -1 });
    match validate(&ctx, &schema, &invalid_age) {
        Ok(()) => println!("unexpected success"),
        Err(e) => println!("invalid:  {invalid_age}  ({e})"),
    }

    // ✗ Invalid: `name` is an integer, not a string.
    let invalid_type = serde_json::json!({ "name": 42, "age": 25 });
    match validate(&ctx, &schema, &invalid_type) {
        Ok(()) => println!("unexpected success"),
        Err(e) => println!("invalid:  {invalid_type}  ({e})"),
    }

    Ok(())
}
