//! # Document Metadata
//!
//! Provides [`DocumentMetadata`], the trait every metadata kind implements.
//!
//! Each document kind keeps its metadata in its own struct with typed fields. The trait gives every
//! kind the same shared fields, so code that reads them works for any kind. The trait cannot check
//! how a struct names its fields on disk, so [`conformance`] checks the serialized shape.
//!
//! ## Modules
//!
//! - [`conformance`]: The check that a metadata kind serializes the shared fields under their exact
//!   names.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::shared::input_document::InputDocument;
use crate::shared::producer::Producer;

pub mod conformance;

/// Version of the shared metadata fields that [`DocumentMetadata`] defines.
pub const METADATA_VERSION: u32 = 1;

/// Defines the fields that the metadata of every stored document carries.
///
/// A store reads the shared fields through this trait for any document kind. Each kind implements
/// it on its own metadata struct, and puts the fields that only this kind has into
/// [`DocumentMetadata::Context`].
///
/// An implementor serializes each shared field at the top level of a JSON object, under the name of
/// its getter. It serializes the context as a nested object under `context`. Every getter returns
/// the value the struct holds, never a constant, so metadata read back from a file reports what the
/// file stores. [`conformance::shape_violations`] checks both rules for one value.
///
/// # Associated Types
///
/// - `Context`: The fields that only this document kind has.
pub trait DocumentMetadata: Serialize + DeserializeOwned + Send + Sync {
    /// The fields that only this document kind has.
    type Context: Serialize + DeserializeOwned;

    /// Names this metadata kind and the version of its context, for example `raw_fetch.v1`.
    ///
    /// A file of this kind stores the same value in its `kind` field.
    const KIND: &'static str;

    /// Returns the version of the shared fields stored in the metadata.
    fn metadata_version(&self) -> u32;

    /// Returns the kind stored in the metadata.
    fn kind(&self) -> &str;

    /// Returns the media type of the document bytes, for example `application/json`.
    fn media_type(&self) -> &str;

    /// Returns the size of the document in bytes.
    fn bytes(&self) -> u64;

    /// Returns the hex-encoded SHA-256 of the document bytes.
    fn sha256(&self) -> &str;

    /// Returns when the producing step made the document bytes.
    fn produced_at(&self) -> DateTime<Utc>;

    /// Returns the id of the run that produced the document bytes.
    fn execution_id(&self) -> Uuid;

    /// Returns the code that wrote the document.
    fn producer(&self) -> &Producer;

    /// Returns the documents this document came from, or an empty slice if it came from none.
    fn inputs(&self) -> &[InputDocument];

    /// Returns the fields that only this document kind has.
    fn context(&self) -> &Self::Context;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::sample_document::SampleMetadata;

    const fn implements_document_metadata<T: DocumentMetadata>() {}
    #[test]
    const fn should_implement_document_metadata_for_sample_metadata() {
        implements_document_metadata::<SampleMetadata>();
    }
}
