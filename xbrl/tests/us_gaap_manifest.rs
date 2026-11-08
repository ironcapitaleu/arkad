//! # US GAAP Manifest Integration Tests
//!
//! Recomputes the `MANIFEST` of each vendored US GAAP release from the files on disk and compares
//! it with the `MANIFEST` in the repository. A changed, added, or removed file fails the test. A
//! second test recomputes the SHA-256 of the `MANIFEST` of the release in use and compares it with
//! the release checksum of the generated tables.
//!
//! ## External Dependencies
//!
//! None. The files live in `taxonomy/us-gaap/<year>/`.

use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;
use sha2::{Digest, Sha256};
use xbrl::us_gaap::taxonomy::RELEASE_CHECKSUM;

const US_GAAP_FOLDER: &str = "taxonomy/us-gaap";
const RELEASE_IN_USE_FOLDER: &str = "taxonomy/us-gaap/2026";
const MANIFEST_FILE_NAME: &str = "MANIFEST";

/// Returns the path of every file below `folder`, relative to `root`, except the `MANIFEST`.
fn vendored_files(root: &Path, folder: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for entry in
        fs::read_dir(folder).expect("Given the vendored release folder, it should be readable")
    {
        let path = entry
            .expect("Given a readable folder, each entry should be readable")
            .path();
        if path.is_dir() {
            files.extend(vendored_files(root, &path));
        } else if path
            .file_name()
            .is_some_and(|name| name != MANIFEST_FILE_NAME)
        {
            let relative = path
                .strip_prefix(root)
                .expect("Given a path found below the root, it should start with the root");
            files.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    files
}

/// Returns the SHA-256 of `bytes` as 64 lowercase hex digits.
fn hex_digest(bytes: &[u8]) -> String {
    let mut hex = String::new();
    for byte in Sha256::digest(bytes) {
        write!(hex, "{byte:02x}").expect("Given a String, a write should never fail");
    }
    hex
}

/// Writes one `MANIFEST` line per file, sorted by path, in the format of `sha256sum`.
fn manifest_of(root: &Path) -> String {
    let mut files = vendored_files(root, root);
    files.sort();
    let mut manifest = String::new();
    for file in files {
        let bytes = fs::read(root.join(&file))
            .expect("Given a listed vendored file, it should be readable");
        let digest = hex_digest(&bytes);
        writeln!(manifest, "{digest}  {file}").expect("Given a String, a write should never fail");
    }
    manifest
}

/// Returns each release folder below `us_gaap`, sorted. Files beside the folders are skipped.
fn release_folders(us_gaap: &Path) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = fs::read_dir(us_gaap)
        .expect("Given the vendored US GAAP folder, it should be readable")
        .map(|entry| {
            entry
                .expect("Given a readable folder, each entry should be readable")
                .path()
        })
        .filter(|path| path.is_dir())
        .collect();
    folders.sort();
    folders
}

#[test]
fn should_match_manifest_for_every_vendored_us_gaap_release() {
    let releases = release_folders(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(US_GAAP_FOLDER));

    let expected_result: Vec<(PathBuf, String)> = releases
        .iter()
        .map(|release| {
            let manifest = fs::read_to_string(release.join(MANIFEST_FILE_NAME))
                .expect("Given a vendored release folder, its MANIFEST should be readable");
            (release.clone(), manifest)
        })
        .collect();

    let result: Vec<(PathBuf, String)> = releases
        .iter()
        .map(|release| (release.clone(), manifest_of(release)))
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_match_release_checksum_for_sha256_of_manifest() {
    let manifest = fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(RELEASE_IN_USE_FOLDER)
            .join(MANIFEST_FILE_NAME),
    )
    .expect("Given the vendored release, its MANIFEST should be readable");

    let expected_result = RELEASE_CHECKSUM;

    let result = hex_digest(&manifest);

    assert_eq!(result, expected_result);
}
