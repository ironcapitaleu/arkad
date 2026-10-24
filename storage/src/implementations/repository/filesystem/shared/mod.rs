//! # Shared
//!
//! Provides the types that more than one module of the
//! [`filesystem`](crate::implementations::repository::filesystem) module uses.
//!
//! ## Modules
//!
//! - [`document_key`]: The [`DocumentKey`](document_key::DocumentKey) naming one document, and the
//!   [`InvalidDocumentKey`](document_key::InvalidDocumentKey) for a path the store rejects.
//! - [`legacy_document_metadata`]: The record of the fetch behind a document, the
//!   [`LegacyDocumentMetadata`](legacy_document_metadata::LegacyDocumentMetadata).
//! - [`raw_document`]: The [`RawDocument`](raw_document::RawDocument) a read returns and a write
//!   accepts.

pub mod document_key;
pub mod legacy_document_metadata;
pub mod raw_document;
