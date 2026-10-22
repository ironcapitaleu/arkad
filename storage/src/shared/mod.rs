//! # Shared
//!
//! Provides the domain types that belong to no single storage backend.
//!
//! These types live outside [`implementations`](crate::implementations), so a second backend can
//! use them without depending on the first.
//!
//! ## Modules
//!
//! - [`document_key`]: The [`DocumentKey`] naming one document, and the [`InvalidDocumentKey`]
//!   for a rejected path.
//! - [`document_metadata`]: The [`DocumentMetadata`] stored beside every document.
//! - [`input_document`]: The [`InputDocument`] pointing at a document that a stored document came
//!   from.
//! - [`producer`]: The [`Producer`] naming the code that wrote a document.

pub mod document_key;
pub mod document_metadata;
pub mod input_document;
pub mod producer;

pub use document_key::{DocumentKey, InvalidDocumentKey};
pub use document_metadata::{DocumentMetadata, METADATA_VERSION};
pub use input_document::InputDocument;
pub use producer::Producer;
