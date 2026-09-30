//! # Document Key
//!
//! Provides [`DocumentKey`], the validated relative path that identifies one document in the store.
//!
//! ## Modules
//!
//! - [`invalid_document_key`]: The [`InvalidDocumentKey`] error for a path the store rejects.
//!
//! ## Usage
//!
//! ```rust
//! use std::path::Path;
//!
//! use storage::implementations::repository::filesystem::DocumentKey;
//!
//! let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
//!     .expect("Given a relative path with no parent component, the key should always build");
//!
//! let expected_result = Path::new("sec/companyfacts/CIK0000320193.json");
//!
//! let result = key.as_path();
//!
//! assert_eq!(result, expected_result);
//! ```

use std::path::{Component, Path, PathBuf};

pub mod invalid_document_key;

pub use invalid_document_key::InvalidDocumentKey;

/// Relative path identifying one document beneath the store's root directory.
///
/// The path is the primary key, so it is also the document's location on disk. Construction
/// drops every `.` component and every repeated separator. It keeps a `~` as an ordinary
/// component, wherever the `~` sits, so `"~/cache/x.json"` names a `~` directory under the root.
/// It does not fold letter case, Unicode form, or the trailing dots and spaces that Windows
/// strips. On a filesystem that ignores those differences, two unequal keys can point to one file.
///
/// Construction also rejects a path that starts at a root directory, and a path that starts with a
/// Windows prefix such as `C:`. It rejects every `..` component, wherever the `..` sits, including
/// one that resolves back inside the root. The check reads the key's text alone, with the separator
/// rules of the platform it runs on. It does not follow symbolic links, so a link inside the root
/// can still point outside it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentKey {
    path: PathBuf,
}

impl DocumentKey {
    /// Creates a new [`DocumentKey`] from a relative path.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidDocumentKey`] if the path:
    /// - names no file below the root ([`InvalidDocumentKey::EmptyPath`]). A path made only of `.`
    ///   components, such as `"."` or `"./"`, names the root itself.
    /// - starts at a root directory ([`InvalidDocumentKey::ContainsRootComponent`]). A leading `\`
    ///   counts only on Windows. On Linux and macOS, `"\data\x.json"` is an ordinary file name and
    ///   builds a key.
    /// - starts with a Windows prefix such as `C:`, on a platform that reads one
    ///   ([`InvalidDocumentKey::ContainsPrefixComponent`]). On Linux and macOS, `"C:x.json"` is an
    ///   ordinary file name and builds a key.
    /// - holds a `..` component ([`InvalidDocumentKey::ContainsParentComponent`]).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::DocumentKey;
    ///
    /// let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
    ///     .expect("Given a valid relative path, the key should always build");
    ///
    /// let expected_result = "CIK0000320193.json";
    ///
    /// let result = key
    ///     .as_path()
    ///     .file_name()
    ///     .expect("Given a path ending in a file name, the file name should always be present")
    ///     .to_string_lossy();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, InvalidDocumentKey> {
        let path = path.into();

        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::RootDir => {
                    return Err(InvalidDocumentKey::contains_root_component(path.clone()));
                }
                Component::Prefix(_) => {
                    return Err(InvalidDocumentKey::contains_prefix_component(path.clone()));
                }
                Component::ParentDir => {
                    return Err(InvalidDocumentKey::contains_parent_component(path.clone()));
                }
                Component::CurDir => {}
                Component::Normal(part) => normalized.push(part),
            }
        }

        if normalized.as_os_str().is_empty() {
            return Err(InvalidDocumentKey::EmptyPath);
        }

        Ok(Self { path: normalized })
    }

    /// Returns the relative path this key names.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_document_key() {
        implements_auto_traits::<DocumentKey>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_document_key() {
        implements_send::<DocumentKey>();
    }

    #[test]
    const fn should_implement_sync_for_document_key() {
        implements_sync::<DocumentKey>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_document_key() {
        implements_debug::<DocumentKey>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_document_key() {
        implements_clone::<DocumentKey>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_document_key() {
        implements_hash::<DocumentKey>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_document_key() {
        implements_eq::<DocumentKey>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_document_key() {
        implements_ord::<DocumentKey>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_document_key() {
        implements_sized::<DocumentKey>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_document_key() {
        implements_partial_eq::<DocumentKey>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_document_key() {
        implements_partial_ord::<DocumentKey>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_document_key() {
        implements_unpin::<DocumentKey>();
    }

    #[test]
    fn should_hold_the_path_when_building_from_a_relative_path() {
        let expected_result = Path::new("sec/companyfacts/CIK0000320193.json");

        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        assert_eq!(key.as_path(), expected_result);
    }

    #[test]
    fn should_reject_an_empty_path_when_building_a_key() {
        let expected_result = InvalidDocumentKey::EmptyPath;

        let result =
            DocumentKey::new("").expect_err("An empty path should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_root_component_when_the_path_starts_with_a_separator() {
        let expected_result = InvalidDocumentKey::contains_root_component("/etc/passwd");

        let result = DocumentKey::new("/etc/passwd")
            .expect_err("A path starting at a root directory should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(windows, ignore = "A backslash separates components only on Windows")]
    fn should_build_a_key_when_a_backslash_path_is_an_ordinary_file_name() {
        let expected_result = Path::new(r"\data\x.json");

        let key = DocumentKey::new(r"\data\x.json").expect(
            "Outside Windows a backslash is an ordinary character, so the key should always build",
        );

        assert_eq!(key.as_path(), expected_result);
    }

    #[test]
    #[cfg_attr(
        not(windows),
        ignore = "A path prefix component exists only on Windows"
    )]
    fn should_reject_a_prefix_component_when_the_path_is_drive_relative() {
        let expected_result = InvalidDocumentKey::contains_prefix_component("C:x.json");

        let result = DocumentKey::new("C:x.json")
            .expect_err("A path resolving against a drive should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(
        not(windows),
        ignore = "A path prefix component exists only on Windows"
    )]
    fn should_reject_a_prefix_component_when_the_path_is_drive_absolute() {
        let expected_result = InvalidDocumentKey::contains_prefix_component(r"C:\data\x.json");

        let result = DocumentKey::new(r"C:\data\x.json")
            .expect_err("A drive-absolute path should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(
        not(windows),
        ignore = "A path prefix component exists only on Windows"
    )]
    fn should_reject_a_prefix_component_when_the_path_names_a_network_share() {
        let expected_result =
            InvalidDocumentKey::contains_prefix_component(r"\\server\share\x.json");

        let result = DocumentKey::new(r"\\server\share\x.json").expect_err(
            "A path resolving against a network share should never build a document key",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_parent_component_when_it_escapes_the_root() {
        let expected_result = InvalidDocumentKey::contains_parent_component("sec/../../etc/passwd");

        let result = DocumentKey::new("sec/../../etc/passwd")
            .expect_err("A path escaping the root should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_parent_component_when_it_resolves_inside_the_root() {
        let expected_result =
            InvalidDocumentKey::contains_parent_component("sec/companyfacts/../submissions/x.json");

        let result = DocumentKey::new("sec/companyfacts/../submissions/x.json")
            .expect_err("A path holding a parent component should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_a_leading_tilde_when_building_a_key() {
        let expected_result = Path::new("~/cache/x.json");

        let key = DocumentKey::new("~/cache/x.json")
            .expect("A tilde is an ordinary component, so the key should always build");

        assert_eq!(key.as_path(), expected_result);
    }

    #[test]
    fn should_drop_a_current_directory_component_when_building_a_key() {
        let expected_result = Path::new("sec/CIK0000320193.json");

        let key = DocumentKey::new("./sec/./CIK0000320193.json")
            .expect("A current directory component stays inside the root, so the key should build");

        assert_eq!(key.as_path(), expected_result);
    }

    #[test]
    fn should_build_equal_keys_when_two_paths_name_the_same_file() {
        let expected_result = DocumentKey::new("sec/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let result = DocumentKey::new("./sec//CIK0000320193.json").expect(
            "Given a relative path with a redundant separator, the key should always build",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_root_path_when_it_is_written_as_a_single_dot() {
        let expected_result = InvalidDocumentKey::EmptyPath;

        let result = DocumentKey::new(".")
            .expect_err("A path naming the root itself should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_root_path_when_it_is_written_with_a_trailing_separator() {
        let expected_result = InvalidDocumentKey::EmptyPath;

        let result = DocumentKey::new("./")
            .expect_err("A path naming the root itself should never build a document key");

        assert_eq!(result, expected_result);
    }
}
