//! # Invalid Document Key
//!
//! Provides [`InvalidDocumentKey`], the error raised when a path cannot name a document.

use std::path::PathBuf;

use thiserror::Error;

/// Error occurring while building a document key.
///
/// Names the kinds of rejected path a caller can act on.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidDocumentKey {
    /// The path names no file below the store's root.
    #[error("[EmptyPath] Document key names no file below the store's root")]
    EmptyPath,

    /// The path starts at a root directory. A leading `/` counts on every platform, and a leading
    /// `\` counts only on Windows.
    #[error(
        "[ContainsRootComponent] Document key '{}' starts at a root directory",
        .path.display()
    )]
    ContainsRootComponent {
        /// The path that was rejected.
        path: PathBuf,
    },

    /// The path starts with a Windows prefix, such as `C:` or `\\server\share`. The path resolves
    /// against that drive or share rather than the store's root, even without a root directory.
    /// Construction reports this variant for a path with both a prefix and a root directory, such
    /// as `C:\data`, because the prefix comes first.
    #[error(
        "[ContainsPrefixComponent] Document key '{}' contains a Windows path prefix",
        .path.display()
    )]
    ContainsPrefixComponent {
        /// The path that was rejected.
        path: PathBuf,
    },

    /// The path holds a `..` component. Construction rejects every `..`, including one that
    /// resolves back inside the store's root.
    #[error(
        "[ContainsParentComponent] Document key '{}' contains a parent component",
        .path.display()
    )]
    ContainsParentComponent {
        /// The path that was rejected.
        path: PathBuf,
    },
}

impl InvalidDocumentKey {
    /// Creates an [`InvalidDocumentKey::ContainsRootComponent`] from the rejected path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::InvalidDocumentKey;
    ///
    /// let error = InvalidDocumentKey::contains_root_component("/etc/passwd");
    ///
    /// let expected_result =
    ///     "[ContainsRootComponent] Document key '/etc/passwd' starts at a root directory";
    ///
    /// let result = error.to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn contains_root_component(path: impl Into<PathBuf>) -> Self {
        Self::ContainsRootComponent { path: path.into() }
    }

    /// Creates an [`InvalidDocumentKey::ContainsPrefixComponent`] from the rejected path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::InvalidDocumentKey;
    ///
    /// let error = InvalidDocumentKey::contains_prefix_component("C:x.json");
    ///
    /// let expected_result =
    ///     "[ContainsPrefixComponent] Document key 'C:x.json' contains a Windows path prefix";
    ///
    /// let result = error.to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn contains_prefix_component(path: impl Into<PathBuf>) -> Self {
        Self::ContainsPrefixComponent { path: path.into() }
    }

    /// Creates an [`InvalidDocumentKey::ContainsParentComponent`] from the rejected path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::InvalidDocumentKey;
    ///
    /// let error = InvalidDocumentKey::contains_parent_component("sec/../etc");
    ///
    /// let expected_result =
    ///     "[ContainsParentComponent] Document key 'sec/../etc' contains a parent component";
    ///
    /// let result = error.to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn contains_parent_component(path: impl Into<PathBuf>) -> Self {
        Self::ContainsParentComponent { path: path.into() }
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
    const fn should_implement_auto_traits_for_invalid_document_key() {
        implements_auto_traits::<InvalidDocumentKey>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_invalid_document_key() {
        implements_send::<InvalidDocumentKey>();
    }

    #[test]
    const fn should_implement_sync_for_invalid_document_key() {
        implements_sync::<InvalidDocumentKey>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_invalid_document_key() {
        implements_debug::<InvalidDocumentKey>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_invalid_document_key() {
        implements_clone::<InvalidDocumentKey>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_invalid_document_key() {
        implements_hash::<InvalidDocumentKey>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_invalid_document_key() {
        implements_eq::<InvalidDocumentKey>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_invalid_document_key() {
        implements_ord::<InvalidDocumentKey>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_for_invalid_document_key() {
        implements_sized::<InvalidDocumentKey>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_invalid_document_key() {
        implements_partial_eq::<InvalidDocumentKey>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_invalid_document_key() {
        implements_partial_ord::<InvalidDocumentKey>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_invalid_document_key() {
        implements_unpin::<InvalidDocumentKey>();
    }

    #[test]
    fn should_build_contains_root_component_variant_when_using_its_constructor() {
        let expected_result = InvalidDocumentKey::ContainsRootComponent {
            path: PathBuf::from("/etc/passwd"),
        };

        let result = InvalidDocumentKey::contains_root_component("/etc/passwd");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_contains_prefix_component_variant_when_using_its_constructor() {
        let expected_result = InvalidDocumentKey::ContainsPrefixComponent {
            path: PathBuf::from("C:x.json"),
        };

        let result = InvalidDocumentKey::contains_prefix_component("C:x.json");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_contains_parent_component_variant_when_using_its_constructor() {
        let expected_result = InvalidDocumentKey::ContainsParentComponent {
            path: PathBuf::from("sec/../etc"),
        };

        let result = InvalidDocumentKey::contains_parent_component("sec/../etc");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_when_the_path_names_no_file() {
        let error = InvalidDocumentKey::EmptyPath;

        let expected_result = "[EmptyPath] Document key names no file below the store's root";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_path_when_path_has_a_root_component() {
        let error = InvalidDocumentKey::contains_root_component("/etc/passwd");

        let expected_result =
            "[ContainsRootComponent] Document key '/etc/passwd' starts at a root directory";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_path_when_path_has_a_prefix_component() {
        let error = InvalidDocumentKey::contains_prefix_component("C:x.json");

        let expected_result =
            "[ContainsPrefixComponent] Document key 'C:x.json' contains a Windows path prefix";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_path_when_path_has_a_parent_component() {
        let error = InvalidDocumentKey::contains_parent_component("sec/../etc");

        let expected_result =
            "[ContainsParentComponent] Document key 'sec/../etc' contains a parent component";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
