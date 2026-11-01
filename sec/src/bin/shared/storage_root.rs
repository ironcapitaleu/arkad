//! # Storage Root
//!
//! Provides the [`RawCompanyFactsWriter`], the [`RawNotesArchiveWriter`], and a writer of any raw
//! source that a binary builds at startup, and the rule that picks their root directory.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::PathBuf;

use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::{RawCompanyFacts, RawNotesArchive, RawSource};
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};

/// The environment variable that names the root directory of the document store.
pub const STORAGE_ROOT_ENV_VAR: &str = "ARKAD_STORAGE_ROOT";

/// The root directory of the document store when `ARKAD_STORAGE_ROOT` is unset or empty.
pub const DEFAULT_STORAGE_ROOT: &str = "data";

/// The storage repository that stores the raw Company Facts of each run.
pub type RawCompanyFactsWriter = FilesystemRepository<Write, RawDocument<RawCompanyFacts>>;

/// The storage repository that stores the Notes files of each run.
// Only `notes_backfill` uses the Notes items. The other binaries include this file, so `dead_code`
// would fire there.
#[allow(dead_code)]
pub type RawNotesArchiveWriter = FilesystemRepository<Write, RawDocument<RawNotesArchive>>;

/// Builds the [`RawCompanyFactsWriter`] for the root that `ARKAD_STORAGE_ROOT` names.
///
/// If the variable is unset or empty, the root is [`DEFAULT_STORAGE_ROOT`]. Building touches no
/// disk.
pub fn raw_company_facts_writer() -> RawCompanyFactsWriter {
    raw_writer()
}

/// Builds the [`RawNotesArchiveWriter`] for the root that `ARKAD_STORAGE_ROOT` names.
///
/// If the variable is unset or empty, the root is [`DEFAULT_STORAGE_ROOT`]. Building touches no
/// disk.
#[allow(dead_code)]
pub fn raw_notes_archive_writer() -> RawNotesArchiveWriter {
    raw_writer()
}

/// Builds a writer of the raw source `S` for the root that `ARKAD_STORAGE_ROOT` names.
///
/// If the variable is unset or empty, the root is [`DEFAULT_STORAGE_ROOT`]. Building touches no
/// disk.
pub fn raw_writer<S: RawSource>() -> FilesystemRepository<Write, RawDocument<S>> {
    FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<S>>()
        .root(storage_root(env::var_os(STORAGE_ROOT_ENV_VAR)))
        .build()
}

/// Creates the root directory of `repository`, with every missing parent.
///
/// The repository does not create its root, so a write under a missing root fails.
///
/// # Errors
///
/// Returns [`io::Error`] if the directory cannot be created.
pub fn create_root<D>(repository: &FilesystemRepository<Write, D>) -> io::Result<()> {
    fs::create_dir_all(repository.root())
}

/// Returns the root that `value` names, or [`DEFAULT_STORAGE_ROOT`] if `value` is unset or empty.
fn storage_root(value: Option<OsString>) -> PathBuf {
    value
        .filter(|root| !root.is_empty())
        .map_or_else(|| PathBuf::from(DEFAULT_STORAGE_ROOT), PathBuf::from)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_default_root_when_variable_is_unset() {
        let value = None;

        let expected_result = PathBuf::from(DEFAULT_STORAGE_ROOT);

        let result = storage_root(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_root_when_variable_is_empty() {
        let value = Some(OsString::new());

        let expected_result = PathBuf::from(DEFAULT_STORAGE_ROOT);

        let result = storage_root(value);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_the_root_directory_when_it_is_missing() {
        let temporary = tempfile::TempDir::new()
            .expect("Given a writable temporary directory, creating a subdirectory should succeed");
        let repository = FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<RawCompanyFacts>>()
            .root(temporary.path().join("nested"))
            .build();

        let expected_result = true;

        create_root(&repository)
            .expect("Given a path under the temporary directory, creating it should succeed");
        let result = repository.root().is_dir();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_given_root_when_variable_is_set() {
        let value = Some(OsString::from("/var/lib/arkad"));

        let expected_result = PathBuf::from("/var/lib/arkad");

        let result = storage_root(value);

        assert_eq!(result, expected_result);
    }
}
