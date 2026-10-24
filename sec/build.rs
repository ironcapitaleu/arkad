//! # Build Script
//!
//! Sets `SEC_GIT_COMMIT` to the git commit of the build, so stored documents can name the code
//! that wrote them. A build outside a git checkout, such as from a crate tarball, sets `unknown`.

use std::process::Command;

fn main() {
    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |commit| commit.trim().to_owned());

    println!("cargo:rustc-env=SEC_GIT_COMMIT={commit}");
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs");
    println!("cargo:rerun-if-changed=../.git/packed-refs");
}
