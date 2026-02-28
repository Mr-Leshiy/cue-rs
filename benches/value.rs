#![allow(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
)]

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use cue_rs::{Ctx, Value};

// ── compile_string ────────────────────────────────────────────────────────────

fn bench_compile_string(c: &mut Criterion) {
    let ctx = Ctx::new().unwrap();

    let mut g = c.benchmark_group("compile_string");

    g.bench_function("int", |b| {
        b.iter(|| Value::compile_string(&ctx, black_box("42")));
    });

    g.bench_function("bool", |b| {
        b.iter(|| Value::compile_string(&ctx, black_box("true")));
    });

    g.bench_function("string", |b| {
        b.iter(|| Value::compile_string(&ctx, black_box(r#""hello world""#)));
    });

    g.bench_function("schema", |b| {
        b.iter(|| {
            Value::compile_string(
                &ctx,
                black_box("{ name: string, age: int & >0, active: bool }"),
            )
        });
    });

    g.finish();
}

// ── compile_bytes ─────────────────────────────────────────────────────────────

fn bench_compile_bytes(c: &mut Criterion) {
    let ctx = Ctx::new().unwrap();

    let mut g = c.benchmark_group("compile_bytes");

    g.bench_function("int", |b| {
        b.iter(|| Value::compile_bytes(&ctx, black_box(b"42")));
    });

    g.bench_function("schema", |b| {
        b.iter(|| {
            Value::compile_bytes(
                &ctx,
                black_box(b"{ name: string, age: int & >0, active: bool }"),
            )
        });
    });

    g.finish();
}

// ── is_valid ──────────────────────────────────────────────────────────────────

fn bench_is_valid(c: &mut Criterion) {
    let ctx = Ctx::new().unwrap();

    let mut g = c.benchmark_group("is_valid");

    g.bench_function("valid_int", |b| {
        b.iter_batched(
            || Value::compile_string(&ctx, "42").unwrap(),
            |v| v.is_valid(),
            BatchSize::SmallInput,
        );
    });

    g.bench_function("valid_schema", |b| {
        b.iter_batched(
            || {
                Value::compile_string(&ctx, "{ name: string, age: int & >0, active: bool }")
                    .unwrap()
            },
            |v| v.is_valid(),
            BatchSize::SmallInput,
        );
    });

    g.finish();
}

// ── unify ─────────────────────────────────────────────────────────────────────

fn bench_unify(c: &mut Criterion) {
    let ctx = Ctx::new().unwrap();

    let mut g = c.benchmark_group("unify");

    g.bench_function("constraint_meets_int", |b| {
        let constraint = Value::compile_string(&ctx, ">0").unwrap();
        let concrete = Value::compile_string(&ctx, "42").unwrap();
        b.iter(|| Value::unify(black_box(&constraint), black_box(&concrete)));
    });

    g.bench_function("schema_meets_value", |b| {
        let schema = Value::compile_string(&ctx, "{ name: string, age: int & >0 }").unwrap();
        let value =
            Value::compile_string(&ctx, r#"{ name: "Alice", age: 30 }"#).unwrap();
        b.iter(|| Value::unify(black_box(&schema), black_box(&value)));
    });

    g.bench_function("incompatible_ints", |b| {
        let a = Value::compile_string(&ctx, "1").unwrap();
        let b_val = Value::compile_string(&ctx, "2").unwrap();
        b.iter(|| Value::unify(black_box(&a), black_box(&b_val)));
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_compile_string,
    bench_compile_bytes,
    bench_is_valid,
    bench_unify,
);
criterion_main!(benches);
