//! Rust bindings for the CUE language runtime, backed by a Go static library.

pub mod ctx;
mod drop;
pub mod error;
pub mod value;

pub use ctx::Ctx;
pub use value::Value;
