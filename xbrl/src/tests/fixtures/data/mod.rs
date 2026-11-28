//! # Fixture Data
//!
//! Real SEC files of 106 filings, and helpers that read and parse them. The 10-Q
//! `0001628280-26-068246` of FEDEX FREIGHT HOLDING COMPANY, INC. is compiled in with
//! `include_str!`, in both sources. The other filings are gzip files that the helpers read at run
//! time.
//!
//! ## Modules
//!
//! - [`instance`]: The instance documents of the filings, as EDGAR serves them.
//! - [`manifest`]: The list of the fixture files, with the source and the SHA-256 of each file.
//! - [`notes`]: The rows of the filings in the Notes files.
//! - [`resolved`]: Parses Notes fixture filings and runs the rules of one filing on them.

use std::fs;
use std::io::Read;
use std::path::Path;

use flate2::read::GzDecoder;

use super::sample_taxonomy::{FakeTaxonomy, declaration, plugin_with_aliases};
use crate::core::taxonomy::reference::ConceptAliases;

pub mod instance;
pub mod manifest;
pub mod notes;
pub mod resolved;

/// The directory that holds the fixture files, at the paths the manifest names.
const DATA_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/tests/fixtures/data");

/// The extension of a Rust source file. A Rust source file is not a fixture file.
const RUST_EXTENSION: &str = "rs";

/// The file name of the manifest. The manifest lists the fixture files and is not one of them.
const MANIFEST_FILE_NAME: &str = "manifest.tsv";

/// The extension of a gzip file.
pub const GZIP_EXTENSION: &str = ".gz";

/// Returns the bytes of the fixture file `file`, a path the manifest names, as stored.
///
/// # Panics
///
/// Panics if the file does not exist.
#[must_use]
pub fn fixture_file(file: &str) -> Vec<u8> {
    let path = Path::new(DATA_DIRECTORY).join(file);
    fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "Given a path the manifest names, the file '{}' should exist: {error}",
            path.display()
        )
    })
}

/// Returns the content of the fixture file `file`: the bytes of a plain file, and the
/// decompressed bytes of a `.gz` file.
///
/// # Panics
///
/// Panics if the file does not exist, or if a `.gz` file is not valid gzip.
#[must_use]
pub fn fixture_content(file: &str) -> Vec<u8> {
    content_of(file, fixture_file(file))
}

/// Returns the content of the fixture file `file` from its bytes as stored, `stored`: the bytes
/// themselves for a plain file, and the decompressed bytes for a `.gz` file.
///
/// # Panics
///
/// Panics if a `.gz` file is not valid gzip.
#[must_use]
pub fn content_of(file: &str, stored: Vec<u8>) -> Vec<u8> {
    if !file.ends_with(GZIP_EXTENSION) {
        return stored;
    }
    let mut content = Vec::new();
    GzDecoder::new(stored.as_slice())
        .read_to_end(&mut content)
        .expect("Given a fixture file written by gzip, the decompression should always succeed");
    content
}

/// Returns the path of every fixture file below the fixture data directory, relative to it, except
/// the Rust sources and the manifest at the root of the directory, in sorted order.
///
/// # Panics
///
/// Panics if a directory below the fixture data directory cannot be read.
#[must_use]
pub fn fixture_files_on_disk() -> Vec<String> {
    let mut files = Vec::new();
    collect_files(Path::new(DATA_DIRECTORY), &mut files);
    files.sort();
    files
}

/// Adds the path of each fixture file below `directory` to `files`, relative to the fixture data
/// directory.
fn collect_files(directory: &Path, files: &mut Vec<String>) {
    let entries = fs::read_dir(directory)
        .expect("Given a directory below the fixture data directory, it should be readable");
    for entry in entries {
        let path = entry
            .expect("Given a readable directory, each entry should be readable")
            .path();
        if path.is_dir() {
            collect_files(&path, files);
        } else {
            let relative = path
                .strip_prefix(DATA_DIRECTORY)
                .expect("Given a path below the fixture data directory, it should start with it");
            let is_rust_source = path
                .extension()
                .is_some_and(|extension| extension == RUST_EXTENSION);
            if !is_rust_source && relative != Path::new(MANIFEST_FILE_NAME) {
                files.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

/// Returns a plug-in that declares `us-gaap`, `dei`, `srt`, and `ecd`, the families of the fixture
/// filing.
#[must_use]
pub fn fixture_taxonomy() -> FakeTaxonomy {
    fixture_taxonomy_with_aliases(Vec::new())
}

/// Returns a plug-in that declares the families of [`fixture_taxonomy`] and holds the alias lists
/// `aliases`.
#[must_use]
pub fn fixture_taxonomy_with_aliases(aliases: Vec<ConceptAliases>) -> FakeTaxonomy {
    plugin_with_aliases(
        "us-gaap/2026",
        vec![
            declaration("us-gaap", "http://fasb.org/us-gaap/{release}", true),
            declaration("dei", "http://xbrl.sec.gov/dei/{release}", true),
            declaration("srt", "http://fasb.org/srt/{release}", true),
            declaration("ecd", "http://xbrl.sec.gov/ecd/{release}", true),
        ],
        aliases,
    )
}
