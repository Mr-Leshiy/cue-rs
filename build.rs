use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=go-cue");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let go_dir = manifest_dir.join("go-cue");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_out = out_dir.join("libgo_cue.a");

    let status = Command::new("go")
        .args([
            "build",
            // 	Build the listed main package, plus all packages it imports,
            // into a C archive file. The only callable symbols will be those
            // functions exported using a cgo //export comment. Requires
            // exactly one main package to be listed.
            "-buildmode=c-archive",
            "-o",
            lib_out.to_str().unwrap(),
            ".",
        ])
        .current_dir(&go_dir)
        .status()
        .expect("failed to run `go build` — is Go installed?");

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
