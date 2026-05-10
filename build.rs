//! Build script for cue-rs: compiles libcue into a static C archive.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::missing_docs_in_private_items
)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn watch_dir(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            watch_dir(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn main() {
    // Docs.rs build, skip everything
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    watch_dir(&manifest_dir.join("libcue"));
    let go_dir = manifest_dir.join("libcue");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let (build_mode, lib_out) = if cfg!(feature = "shared") {
        ("-buildmode=c-shared", out_dir.join("libcue.so"))
    } else {
        ("-buildmode=c-archive", out_dir.join("libcue.a"))
    };

    let status = Command::new("go")
        .args([
            "build",
            build_mode,
            "-o",
            lib_out.to_str().expect("lib_out path is not valid UTF-8"),
            "github.com/cue-lang/libcue",
        ])
        .current_dir(&go_dir)
        .status()
        .expect("failed to run go build");

    assert!(status.success(), "go build failed");

    println!("cargo:rustc-link-search=all={}", out_dir.display());
    if cfg!(feature = "shared") {
        println!("cargo:rustc-link-lib=dylib=cue");
    } else {
        println!("cargo:rustc-link-lib=static=cue");
    }

    // The Go runtime leaves platform system-library symbols unresolved in the
    // static archive; the final Rust linker must supply them.
    if cfg!(target_os = "macos") {
        // CoreFoundation / Security are used by Go's crypto/tls and net packages.
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");
    }
}
