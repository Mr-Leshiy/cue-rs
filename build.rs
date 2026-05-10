//! Build script for cue-rs: compiles libcue into a static C archive.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{env, path::PathBuf, process::Command};

fn main() {
    // Docs.rs build, skip everything
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    // Rebuild whenever the module manifest or lockfile changes (i.e. a version
    // bump of github.com/cue-lang/libcue).
    println!("cargo:rerun-if-changed=go-cue/go.mod");
    println!("cargo:rerun-if-changed=go-cue/go.sum");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let go_dir = manifest_dir.join("libcue");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_out = out_dir.join("libcue.a");

    let mut cmd = Command::new("go");
    cmd.args([
        "build",
        // Build the external module (github.com/cue-lang/libcue), which
        // declares `package main` and exports C symbols via cgo, into a
        // static C archive.
        "-buildmode=c-archive",
        "-o",
        lib_out.to_str().expect("lib_out path is not valid UTF-8"),
        "github.com/cue-lang/libcue",
    ])
    .current_dir(&go_dir);

    // When targeting musl, CGO must use musl's C compiler so that libcue.a is
    // compiled against musl libc rather than glibc. Respect Cargo's per-target
    // CC override convention (CC_<target>) before falling back to musl-gcc.
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if &target_env == "musl" {
        cmd.env("CC", "musl-gcc");
    }


    let status = cmd.status().expect("failed to run go build");

    assert!(status.success(), "go build failed");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=cue");

    // The Go runtime leaves platform system-library symbols unresolved in the
    // static archive; the final Rust linker must supply them.
    if cfg!(target_os = "macos") {
        // CoreFoundation / Security are used by Go's crypto/tls and net packages.
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");
    }
}
