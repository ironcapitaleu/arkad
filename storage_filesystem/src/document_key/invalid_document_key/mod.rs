//! # Invalid Document Key
//!
//! Provides [`InvalidDocumentKey`], the error raised when a path cannot name a document.
//!
//! ## Usage
//!
//! ```rust
//! use storage_filesystem::InvalidDocumentKey;
//!
//! let _err = InvalidDocumentKey::absolute_path("/etc/passwd");
//! ```

use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Error occurring while building a document key.
///
/// Separates the different kinds of rejected path so a caller can tell them apart.
pub enum InvalidDocumentKey {
    /// The path holds no components.
    #[error("[EmptyPath] Document key is empty")]
    EmptyPath,

    /// The path is absolute, so it resolves outside the store's root.
    #[error("[AbsolutePath] Document key '{path}' is absolute, but must be relative")]
    AbsolutePath {
        /// The path that was rejected.
        path: String,
    },

    /// The path holds a `..` component, so it can resolve outside the store's root.
    #[error("[ContainsParentComponent] Document key '{path}' contains a parent component")]
    ContainsParentComponent {
        /// The path that was rejected.
        path: String,
    },
}

impl InvalidDocumentKey {
    /// Creates an [`InvalidDocumentKey::AbsolutePath`] from the rejected path.
    #[must_use]
    pub fn absolute_path(path: impl Into<String>) -> Self {
        Self::AbsolutePath { path: path.into() }
    }

    /// Creates an [`InvalidDocumentKey::ContainsParentComponent`] from the rejected path.
    #[must_use]
    pub fn contains_parent_component(path: impl Into<String>) -> Self {
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
    const fn should_implement_auto_traits_when_using_invalid_document_key() {
        implements_auto_traits::<InvalidDocumentKey>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_invalid_document_key() {
        implements_send::<InvalidDocumentKey>();
    }

    #[test]
    const fn should_implement_sync_when_using_invalid_document_key() {
        implements_sync::<InvalidDocumentKey>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_invalid_document_key() {
        implements_debug::<InvalidDocumentKey>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_invalid_document_key() {
        implements_clone::<InvalidDocumentKey>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_invalid_document_key() {
        implements_hash::<InvalidDocumentKey>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_invalid_document_key() {
        implements_eq::<InvalidDocumentKey>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_invalid_document_key() {
        implements_ord::<InvalidDocumentKey>();
    }

    #[test]
    fn should_build_absolute_path_variant_when_using_its_constructor() {
        let expected_result = InvalidDocumentKey::AbsolutePath {
            path: "/etc/passwd".to_owned(),
        };

        let result = InvalidDocumentKey::absolute_path("/etc/passwd");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_contains_parent_component_variant_when_using_its_constructor() {
        let expected_result = InvalidDocumentKey::ContainsParentComponent {
            path: "sec/../etc".to_owned(),
        };

        let result = InvalidDocumentKey::contains_parent_component("sec/../etc");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_when_path_is_empty() {
        let error = InvalidDocumentKey::EmptyPath;

        let expected_result = "[EmptyPath] Document key is empty";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_path_when_path_is_absolute() {
        let error = InvalidDocumentKey::absolute_path("/etc/passwd");

        let expected_result =
            "[AbsolutePath] Document key '/etc/passwd' is absolute, but must be relative";

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
