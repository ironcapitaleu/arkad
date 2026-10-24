//! # Build Script
//!
//! Sets `SEC_GIT_COMMIT` to the git commit of the build, so stored documents can name the code
//! that wrote them. A build outside a git checkout of this repository, such as from a packaged
//! crate, sets `unknown`.

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Given a cargo build script, CARGO_MANIFEST_DIR should always be set");
    // A packaged crate holds `.cargo_vcs_info.json`. Its directory can sit inside an unrelated git
    // repository, whose commit would name the wrong build.
    let is_packaged = Path::new(&manifest_dir)
        .join(".cargo_vcs_info.json")
        .exists();
    let commit = if is_packaged {
        None
    } else {
        git(&["rev-parse", "HEAD"])
    };
    if commit.is_none() {
        println!("cargo:warning=no git checkout found, SEC_GIT_COMMIT is 'unknown'");
    }
    println!(
        "cargo:rustc-env=SEC_GIT_COMMIT={}",
        commit.as_deref().unwrap_or("unknown")
    );

    println!("cargo:rerun-if-changed=build.rs");
    // Cargo reruns a script on every build if a watched path does not exist, so the script asks
    // git for the paths of this checkout, which differ in a worktree, and watches only those. It
    // watches `refs/heads`, not `refs`, because a fetch or a new tag changes no commit of the
    // checkout. A repository in the reftable format keeps its refs in `reftable`.
    for name in ["HEAD", "refs/heads", "packed-refs", "reftable"] {
        if let Some(path) = git(&["rev-parse", "--git-path", name])
            && Path::new(&path).exists()
        {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}

/// Runs git with the given arguments, and returns its trimmed output if it succeeds.
fn git(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_owned())
}
