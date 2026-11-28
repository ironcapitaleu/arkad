//! # Source Document
//!
//! Provides [`SourceDocument`], the raw document that the rows of one filing come from.

use storage::implementations::repository::filesystem::{DocumentKey, IngestionMode};
use storage::shared::Sha256Digest;

use crate::datasets::SourceKind;

/// The raw document of a row, for the ingestion provenance of each row of `as_is_facts` and
/// `filings`.
///
/// A unit reader fills it once per Notes file, and once per instance of a filing day. It takes
/// `ingestion_mode` from the raw metadata of the document.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceDocument {
    /// The kind of the raw document.
    pub kind: SourceKind,
    /// The key of the raw document in the document store.
    pub key: DocumentKey,
    /// The SHA-256 of the raw document.
    pub sha256: Sha256Digest,
    /// The ingestion mode of the raw document, from its metadata.
    pub ingestion_mode: IngestionMode,
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_source_document() {
        implements_auto_traits::<SourceDocument>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_source_document() {
        implements_value_traits::<SourceDocument>();
    }
}
