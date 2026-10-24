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

use serde::{Deserialize, Serialize, Serializer};

pub mod invalid_document_key;

pub use invalid_document_key::InvalidDocumentKey;

/// Relative path identifying one document beneath the store's root directory.
///
/// The path of the document serves as the primary key. Construction
/// drops every `.` component and every repeated separator. It rebuilds the path with the
/// separator of its platform, so `"sec/x.json"` holds a `\` on Windows. It does not fold letter
/// case, Unicode form, or the trailing dots and spaces that Windows strips. On a filesystem that
/// ignores those differences, two unequal keys can point to one file.
///
/// Construction also rejects a path that starts at a root directory, and a path that starts with a
/// Windows prefix such as `C:`. It rejects every `..` component, wherever the `..` sits, including
/// one that resolves back inside the root. The check reads the key's text alone, with the separator
/// rules of the platform it runs on. It does not follow symbolic links, so a link inside the root
/// can still point outside it.
///
/// Construction rejects a component that holds a `\` or a `:`, or that is not valid UTF-8. So a key
/// names the same file on every platform. A key serializes as one string with `/` between its
/// components. Deserializing goes through [`DocumentKey::new`], so a key read from a file passes
/// the same checks.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct DocumentKey {
    path: PathBuf,
}

impl DocumentKey {
    /// Creates a new [`DocumentKey`] from a relative path.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidDocumentKey`] if the path:
    /// - starts with a Windows prefix such as `C:` or `\\server\share`, on Windows
    ///   ([`InvalidDocumentKey::ContainsPrefixComponent`]).
    /// - starts at a root directory ([`InvalidDocumentKey::ContainsRootComponent`]). A leading `/`
    ///   names a root on every platform, and a single leading `\` names one only on Windows.
    /// - holds a `..` component ([`InvalidDocumentKey::ContainsParentComponent`]).
    /// - holds a component that is not valid UTF-8
    ///   ([`InvalidDocumentKey::ContainsNonUtf8Component`]).
    /// - holds a component with a `\` or a `:`
    ///   ([`InvalidDocumentKey::ContainsNonPortableComponent`]). Only Linux and macOS read such a
    ///   component, so `"C:x.json"`, `"\data\x.json"`, and `"sec\..\x.json"` report this variant
    ///   there and a prefix, a root, or a parent on Windows.
    /// - names no file below the root ([`InvalidDocumentKey::EmptyPath`]). A path made only of `.`
    ///   components, such as `"."` or `"./"`, names the root itself.
    ///
    /// Construction walks the path's components in order and reports the first one it rejects, so
    /// `"/sec/../x.json"` reports the root rather than the `..`. On Windows, `"C:\data"` reports
    /// the prefix rather than the root. A path whose components are all accepted, and that still
    /// names no file, reports [`InvalidDocumentKey::EmptyPath`].
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
                Component::Normal(part) => {
                    let Some(text) = part.to_str() else {
                        return Err(InvalidDocumentKey::contains_non_utf8_component(
                            path.clone(),
                        ));
                    };
                    if text.contains(['\\', ':']) {
                        return Err(InvalidDocumentKey::contains_non_portable_component(
                            path.clone(),
                        ));
                    }
                    normalized.push(part);
                }
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

impl TryFrom<String> for DocumentKey {
    type Error = InvalidDocumentKey;

    /// Converts a path string into a [`DocumentKey`], with the checks of [`DocumentKey::new`].
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidDocumentKey`] if the path cannot name a document.
    fn try_from(path: String) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}

impl Serialize for DocumentKey {
    /// Serializes the key as one string, with `/` between its components.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let components: Vec<&str> = self
            .path
            .components()
            .map(|component| {
                component.as_os_str().to_str().expect(
                    "Given a built key, construction rejected every component that is not valid UTF-8, so the conversion should always succeed",
                )
            })
            .collect();
        serializer.serialize_str(&components.join("/"))
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
        let path = "sec/companyfacts/CIK0000320193.json";

        let expected_result = Path::new("sec/companyfacts/CIK0000320193.json");

        let key = DocumentKey::new(path)
            .expect("Given a valid relative path, the key should always build");
        let result = key.as_path();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_an_empty_path_when_building_a_key() {
        let path = "";

        let expected_result = InvalidDocumentKey::EmptyPath;

        let result =
            DocumentKey::new(path).expect_err("An empty path should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_root_component_when_the_path_starts_with_a_forward_slash() {
        let path = "/etc/passwd";

        let expected_result = InvalidDocumentKey::contains_root_component("/etc/passwd");

        let result = DocumentKey::new(path)
            .expect_err("A path starting at a root directory should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(windows, ignore = "A backslash separates components only on Windows")]
    fn should_reject_a_non_portable_component_when_the_path_starts_with_a_backslash() {
        let path = r"\data\x.json";

        let expected_result = InvalidDocumentKey::contains_non_portable_component(r"\data\x.json");

        let result = DocumentKey::new(path).expect_err(
            "Outside Windows a backslash stays inside a component, so the key should never build",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(windows, ignore = "A backslash separates components only on Windows")]
    fn should_reject_a_non_portable_component_when_a_component_holds_a_backslash() {
        let path = r"sec\x.json";

        let expected_result = InvalidDocumentKey::contains_non_portable_component(r"sec\x.json");

        let result = DocumentKey::new(path).expect_err(
            "Outside Windows a backslash stays inside a component, so the key should never build",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(windows, ignore = "A leading drive letter is a prefix on Windows")]
    fn should_reject_a_non_portable_component_when_the_path_starts_with_a_drive_letter() {
        let path = "C:x.json";

        let expected_result = InvalidDocumentKey::contains_non_portable_component("C:x.json");

        let result = DocumentKey::new(path).expect_err(
            "Outside Windows a drive letter stays inside a component, so the key should never build",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_non_portable_component_when_a_later_component_holds_a_colon() {
        let path = "sec/a:b.json";

        let expected_result = InvalidDocumentKey::contains_non_portable_component("sec/a:b.json");

        let result = DocumentKey::new(path)
            .expect_err("A component holding a colon should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[cfg(unix)]
    #[test]
    fn should_reject_a_non_utf8_component_when_building_a_key() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let path = OsStr::from_bytes(b"sec/\xFF.json");

        let expected_result = InvalidDocumentKey::contains_non_utf8_component(path);

        let result = DocumentKey::new(path)
            .expect_err("A component that is not valid UTF-8 should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(
        not(windows),
        ignore = "A path prefix component exists only on Windows"
    )]
    fn should_reject_a_prefix_component_when_the_path_is_drive_relative() {
        let path = "C:x.json";

        let expected_result = InvalidDocumentKey::contains_prefix_component("C:x.json");

        let result = DocumentKey::new(path)
            .expect_err("A path resolving against a drive should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(
        not(windows),
        ignore = "A path prefix component exists only on Windows"
    )]
    fn should_reject_a_prefix_component_when_the_path_is_drive_absolute() {
        let path = r"C:\data\x.json";

        let expected_result = InvalidDocumentKey::contains_prefix_component(r"C:\data\x.json");

        let result = DocumentKey::new(path)
            .expect_err("A drive-absolute path should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    #[cfg_attr(
        not(windows),
        ignore = "A path prefix component exists only on Windows"
    )]
    fn should_reject_a_prefix_component_when_the_path_names_a_network_share() {
        let path = r"\\server\share\x.json";

        let expected_result =
            InvalidDocumentKey::contains_prefix_component(r"\\server\share\x.json");

        let result = DocumentKey::new(path).expect_err(
            "A path resolving against a network share should never build a document key",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_root_component_when_it_comes_before_a_parent_component() {
        let path = "/sec/../x.json";

        let expected_result = InvalidDocumentKey::contains_root_component("/sec/../x.json");

        let result = DocumentKey::new(path).expect_err(
            "A path starting at a root directory should never build a document key, whatever else \
             it holds",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_parent_component_when_no_other_component_survives() {
        let path = "./..";

        let expected_result = InvalidDocumentKey::contains_parent_component("./..");

        let result = DocumentKey::new(path)
            .expect_err("A path naming the parent of the root should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_parent_component_when_it_escapes_the_root() {
        let path = "sec/../../etc/passwd";

        let expected_result = InvalidDocumentKey::contains_parent_component("sec/../../etc/passwd");

        let result = DocumentKey::new(path)
            .expect_err("A path escaping the root should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_parent_component_when_it_resolves_inside_the_root() {
        let path = "sec/companyfacts/../submissions/x.json";

        let expected_result =
            InvalidDocumentKey::contains_parent_component("sec/companyfacts/../submissions/x.json");

        let result = DocumentKey::new(path)
            .expect_err("A path holding a parent component should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_a_current_directory_component_when_building_a_key() {
        let path = "./sec/./CIK0000320193.json";

        let expected_result = Path::new("sec/CIK0000320193.json");

        let key = DocumentKey::new(path)
            .expect("A current directory component stays inside the root, so the key should build");
        let result = key.as_path();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_equal_keys_when_two_paths_name_the_same_file() {
        let plain_path = "sec/CIK0000320193.json";
        let redundant_path = "./sec//CIK0000320193.json";

        let expected_result = DocumentKey::new(plain_path)
            .expect("Given a valid relative path, the key should always build");

        let result = DocumentKey::new(redundant_path).expect(
            "Given a relative path with a redundant separator, the key should always build",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_root_path_when_it_is_written_as_a_single_dot() {
        let path = ".";

        let expected_result = InvalidDocumentKey::EmptyPath;

        let result = DocumentKey::new(path)
            .expect_err("A path naming the root itself should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_root_path_when_it_is_written_with_a_trailing_separator() {
        let path = "./";

        let expected_result = InvalidDocumentKey::EmptyPath;

        let result = DocumentKey::new(path)
            .expect_err("A path naming the root itself should never build a document key");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_key_as_one_string_for_a_nested_key() {
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let expected_result = r#""sec/companyfacts/CIK0000320193.json""#;

        let result = serde_json::to_string(&key)
            .expect("Given a key of UTF-8 components, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_the_same_key_when_deserializing_a_serialized_key() {
        let key = DocumentKey::new("sec/companyfacts/CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");
        let json = serde_json::to_string(&key)
            .expect("Given a key of UTF-8 components, serialization should always succeed");

        let expected_result = key;

        let result: DocumentKey = serde_json::from_str(&json)
            .expect("Given JSON this type produced, deserialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_parent_component_when_deserializing_a_key() {
        let json = r#""sec/../CIK0000320193.json""#;

        let expected_result = "[ContainsParentComponent] Document key 'sec/../CIK0000320193.json' contains a parent component";

        let result = serde_json::from_str::<DocumentKey>(json)
            .expect_err("Given a path with a parent component, deserialization should always fail")
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[cfg(windows)]
    #[test]
    fn should_serialize_with_forward_slashes_when_running_on_windows() {
        let key = DocumentKey::new(r"sec\companyfacts\CIK0000320193.json")
            .expect("Given a valid relative path, the key should always build");

        let expected_result = r#""sec/companyfacts/CIK0000320193.json""#;

        let result = serde_json::to_string(&key)
            .expect("Given a key of UTF-8 components, serialization should always succeed");

        assert_eq!(result, expected_result);
    }
}
