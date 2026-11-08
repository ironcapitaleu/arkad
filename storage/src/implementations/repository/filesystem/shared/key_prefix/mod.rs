//! # Key Prefix
//!
//! Provides [`KeyPrefix`], the directory under the root that a listing selects.

use std::path::Path;

use crate::implementations::repository::filesystem::shared::document_key::DocumentKey;

/// Selects the keys under one directory of the root, or every key, and optionally only the keys
/// after a cursor.
///
/// The prefix matches whole path components. So the prefix `sec/notes` selects
/// `sec/notes/2024q1_notes.zip`, not `sec/notes-old/a.zip`. A listing starts at the directory the
/// prefix names and reads nothing above it. [`KeyPrefix::root`] selects every key.
///
/// [`KeyPrefix::starting_after`] sets a cursor: the listing then yields only the keys that sort
/// after it, by the bytes of the key text. A caller that keeps the last key it finished resumes a
/// listing from there. The cursor need not name a stored key.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
///
/// use storage::implementations::repository::filesystem::{DocumentKey, KeyPrefix};
///
/// let directory = DocumentKey::new("sec/notes")
///     .expect("Given a valid relative path, the key should always build");
///
/// let expected_result = Path::new("sec/notes");
///
/// let prefix = KeyPrefix::from(directory);
/// let result = prefix.as_path();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyPrefix {
    directory: Option<DocumentKey>,
    start_after: Option<DocumentKey>,
}

impl KeyPrefix {
    /// Creates a [`KeyPrefix`] that selects every key under the root.
    #[must_use]
    pub const fn root() -> Self {
        Self {
            directory: None,
            start_after: None,
        }
    }

    /// Returns the prefix with a cursor, so a listing yields only the keys that sort after it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::implementations::repository::filesystem::{DocumentKey, KeyPrefix};
    ///
    /// let last_key = DocumentKey::new("sec/notes/2024q1_notes.zip")
    ///     .expect("Given a valid relative path, the key should always build");
    ///
    /// let expected_result = Some(&last_key);
    ///
    /// let prefix = KeyPrefix::root().starting_after(last_key.clone());
    /// let result = prefix.start_after();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn starting_after(self, key: DocumentKey) -> Self {
        Self {
            start_after: Some(key),
            ..self
        }
    }

    /// Returns the cursor, if the prefix has one.
    #[must_use]
    pub const fn start_after(&self) -> Option<&DocumentKey> {
        self.start_after.as_ref()
    }

    /// Returns the directory the prefix names, relative to the root.
    ///
    /// The path is empty for [`KeyPrefix::root`].
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.directory
            .as_ref()
            .map_or_else(|| Path::new(""), DocumentKey::as_path)
    }
}

impl From<DocumentKey> for KeyPrefix {
    /// Creates a [`KeyPrefix`] that selects the keys under the directory the key names.
    fn from(directory: DocumentKey) -> Self {
        Self {
            directory: Some(directory),
            start_after: None,
        }
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
    const fn should_implement_auto_traits_for_key_prefix() {
        implements_auto_traits::<KeyPrefix>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Default + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_key_prefix() {
        implements_value_traits::<KeyPrefix>();
    }

    #[test]
    fn should_return_an_empty_path_when_the_prefix_selects_the_root() {
        let prefix = KeyPrefix::root();

        let expected_result = Path::new("");

        let result = prefix.as_path();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_select_the_root_when_the_prefix_is_the_default() {
        let prefix = KeyPrefix::default();

        let expected_result = KeyPrefix::root();

        let result = prefix;

        assert_eq!(result, expected_result);
    }
}
