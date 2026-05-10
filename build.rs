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
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    watch_dir(&manifest_dir.join("libcue"));
    let go_dir = manifest_dir.join("libcue");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // CARGO_CFG_TARGET_OS reflects the actual cargo `--target`, unlike
    // `cfg!(target_os)` which reflects the host of the build script.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let shared = env::var("CARGO_FEATURE_SHARED").is_ok();

    let lib_filename = match (shared, target_os.as_str()) {
        (false, _) => "libcue.a",
        (true, "macos") => "libcue.dylib",
        (true, _) => "libcue.so",
    };
    let lib_out = out_dir.join(lib_filename);

    let build_mode = if shared {
        "-buildmode=c-shared"
    } else {
        "-buildmode=c-archive"
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

    println!("cargo:rustc-link-search=native={}", out_dir.display());

    if shared {
        link_shared(&out_dir, lib_filename, &target_os);
    } else {
        println!("cargo:rustc-link-lib=static=cue");
    }

    // The Go runtime leaves platform system-library symbols unresolved; the
    // final Rust linker must supply them (in both link modes).
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");
    }
}

fn link_shared(
    out_dir: &Path,
    lib_filename: &str,
    _target_os: &str,
) {
    println!("cargo:rustc-link-lib=dylib=cue");

    // Copy the shared lib to where every binary that links cue-rs will be
    // produced.  Without this, doctest binaries (compiled into a fresh
    // `/tmp/rustdoctestXXXX/rust_out`) and tests under `target/.../deps/`
    // cannot find `libcue.{so,dylib}` at runtime — only the absolute
    // rpath baked below saves them.
    let target_dir = out_dir
        .ancestors()
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == "debug" || n == "release")
        })
        .expect("OUT_DIR is not under target/<...>/<profile>/");
    let src = out_dir.join(lib_filename);
    fs::copy(&src, target_dir.join(lib_filename)).unwrap();

    // Absolute rpath into the deps dir.  Doctest binaries live in a
    // throwaway tmp directory so $ORIGIN cannot reach them; the absolute
    // path resolves regardless of where the binary runs from.  This is a
    // dev/CI convenience — it bakes a build-machine path into the binary,
    // so consumer crates that ship binaries should not enable `shared`.
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", target_dir.display());
}
