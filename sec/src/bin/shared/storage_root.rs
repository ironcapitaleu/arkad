//! # Storage Root
//!
//! Provides the [`RawCompanyFactsWriter`] that a binary builds at startup, and the rule that picks
//! its root directory.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::PathBuf;

use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawCompanyFacts;
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};

/// The environment variable that names the root directory of the document store.
pub const STORAGE_ROOT_ENV_VAR: &str = "ARKAD_STORAGE_ROOT";

/// The root directory of the document store when `ARKAD_STORAGE_ROOT` is unset or empty.
pub const DEFAULT_STORAGE_ROOT: &str = "data";

/// The storage repository that stores the raw Company Facts of each run.
pub type RawCompanyFactsWriter = FilesystemRepository<Write, RawDocument<RawCompanyFacts>>;

/// Builds the [`RawCompanyFactsWriter`] for the root that `ARKAD_STORAGE_ROOT` names.
///
/// If the variable is unset or empty, the root is [`DEFAULT_STORAGE_ROOT`]. Building touches no
/// disk.
pub fn raw_company_facts_writer() -> RawCompanyFactsWriter {
    FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<RawCompanyFacts>>()
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
pub fn create_root(repository: &RawCompanyFactsWriter) -> io::Result<()> {
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
        let expected_result = PathBuf::from(DEFAULT_STORAGE_ROOT);

        let result = storage_root(None);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_root_when_variable_is_empty() {
        let expected_result = PathBuf::from(DEFAULT_STORAGE_ROOT);

        let result = storage_root(Some(OsString::new()));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_the_root_directory_when_it_is_missing() {
        let root = env::temp_dir().join(format!("arkad-storage-root-{}", std::process::id()));
        let repository = FilesystemRepository::builder()
            .writer()
            .document_type::<RawDocument<RawCompanyFacts>>()
            .root(root.join("nested"))
            .build();

        let expected_result = true;

        create_root(&repository)
            .expect("Given a path under the temporary directory, creating it should succeed");
        let result = repository.root().is_dir();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_given_root_when_variable_is_set() {
        let expected_result = PathBuf::from("/var/lib/arkad");

        let result = storage_root(Some(OsString::from("/var/lib/arkad")));

        assert_eq!(result, expected_result);
    }
}
