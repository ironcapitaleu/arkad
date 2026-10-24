//! # Shared
//!
//! Provides the types that more than one module of the
//! [`filesystem`](crate::implementations::repository::filesystem) module uses.
//!
//! ## Modules
//!
//! - [`document_key`]: The [`DocumentKey`](document_key::DocumentKey) naming one document.
//! - [`document_metadata`]: The [`DocumentMetadata`](document_metadata::DocumentMetadata)
//!   describing the fetch behind a document.
//! - [`raw_document`]: The [`RawDocument`](raw_document::RawDocument) a read returns and a write
//!   accepts.

pub mod document_key;
pub mod document_metadata;
pub mod raw_document;
