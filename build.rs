//! Build script for cue-rs: compiles libcue into a static C archive.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{env, path::PathBuf, process::Command};

fn main() {
    // Rebuild whenever the module manifest or lockfile changes (i.e. a version
    // bump of github.com/cue-lang/libcue).
    println!("cargo:rerun-if-changed=go-cue/go.mod");
    println!("cargo:rerun-if-changed=go-cue/go.sum");

    // docs.rs sets this env var; skip the Go build since it has no Go toolchain.
    // cargo doc does not link, so omitting the link directives is safe.
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let go_dir = manifest_dir.join("libcue");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_out = out_dir.join("libcue.a");

    let status = Command::new("go")
        .args([
            "build",
            // Build the external module (github.com/cue-lang/libcue), which
            // declares `package main` and exports C symbols via cgo, into a
            // static C archive.
            "-buildmode=c-archive",
            "-o",
            lib_out.to_str().expect("lib_out path is not valid UTF-8"),
            "github.com/cue-lang/libcue",
        ])
        .current_dir(&go_dir)
        .status()
        .expect("failed to run go build");

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
