# cue-rs

Rust bindings for the [CUE](https://cuelang.org) language runtime.

The CUE evaluation engine is written in Go with the C interface. Staticly linked with the `Rust` code, exposing a safe `Rust` API on top.

```mermaid
flowchart LR
    Go["Go (CUE runtime)\ncuelang.org/go"]
    C["C interface\n(cgo export)"]
    Rust["Rust\nsafe API"]

    Go -->|"compiled to\nlibgo_cue.a"| C
    C -->|"statically linked"| Rust
```

## Requirements

- **Go 1.21+** — the Go toolchain is required.

## Usage

```rust
use cue_rs::value::Value;

let v = Value::new(r#"{ name: "alice", age: 30 }"#).unwrap();

println!("{}", v.to_json_string().unwrap());
println!("{}", v.to_yaml_string().unwrap());
```
