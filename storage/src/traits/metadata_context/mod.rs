//! # Metadata Context
//!
//! Provides [`MetadataContext`], the trait every document kind implements for its metadata.

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Defines the fields that only one document kind stores in its metadata.
///
/// Every stored document has the same shared metadata fields. A document kind adds its own fields
/// in one context type, and the context type names the kind through [`MetadataContext::KIND`].
/// [`DocumentMetadata`](crate::DocumentMetadata) holds the shared fields and one context.
pub trait MetadataContext: Serialize + DeserializeOwned {
    /// Names the document kind and the version of its context, for example `raw_fetch.v1`.
    ///
    /// Raise the version when a field of the context is removed, renamed, or redefined.
    const KIND: &'static str;
}
