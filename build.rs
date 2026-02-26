//! Build script for cue-rs: compiles the Go CUE library into a static C archive.

use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=go-cue");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let go_dir = manifest_dir.join("go-cue");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_out = out_dir.join("libgo_cue.a");

    let mut cmd = Command::new("go");
    cmd.arg("build");

    // When targeting musl, use musl-gcc so CGo compiles against musl headers
    // rather than glibc's, which avoids unresolved references to glibc-specific
    // fortified symbols.
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("musl") {
        cmd.env("CC", "musl-gcc");
        cmd.args(["-ldflags", "-linkmode external -extldflags '--static-pie'"]);
    }

    cmd.args([
        // Build the listed main package, plus all packages it imports,
        // into a C archive file. The only callable symbols will be those
        // functions exported using a cgo //export comment. Requires
        // exactly one main package to be listed.
        "-buildmode=c-archive",
        "-o",
        lib_out.to_str().expect("lib_out path is not valid UTF-8"),
        ".",
    ])
    .current_dir(&go_dir);

    let status = cmd.status().expect("failed to run go build");

    assert!(status.success(), "go build failed");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=go_cue");

    // The Go runtime leaves platform system-library symbols unresolved in the
    // static archive; the final Rust linker must supply them.
    if cfg!(target_os = "macos") {
        // CoreFoundation / Security are used by Go's crypto/tls and net packages.
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");
    }
}
