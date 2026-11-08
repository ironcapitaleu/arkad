//! # Stored Document
//!
//! Provides [`StoredDocument`], what a listing found in place of the document file of a key.

/// What a listing found in place of the document file of a key.
///
/// A write renames the metadata into place before the document. So a listing that runs during a
/// write can find the metadata of a new key without its document, or new metadata beside the old
/// document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StoredDocument {
    /// The document file exists, with this size on disk.
    Present {
        /// The number of bytes of the document file.
        size_in_bytes: u64,
    },
    /// The metadata file exists, but the document file does not.
    Missing,
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_stored_document() {
        implements_auto_traits::<StoredDocument>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_stored_document() {
        implements_value_traits::<StoredDocument>();
    }
}
