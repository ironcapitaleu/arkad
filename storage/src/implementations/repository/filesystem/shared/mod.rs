//! # Shared
//!
//! Provides the types that more than one module of the
//! [`filesystem`](crate::implementations::repository::filesystem) module uses.
//!
//! ## Modules
//!
//! - [`document_key`]: The [`DocumentKey`](document_key::DocumentKey) naming one document, and the
//!   [`InvalidDocumentKey`](document_key::InvalidDocumentKey) for a path the store rejects.
//! - [`document_metadata`]: The parts of the metadata that every stored document carries, such
//!   as the [`Producer`](document_metadata::Producer).
//! - [`key_prefix`]: The [`KeyPrefix`](key_prefix::KeyPrefix) that selects the keys a listing
//!   walks.
//! - [`listed_document`]: The [`ListedDocument`](listed_document::ListedDocument) that a listing
//!   yields for one document.
//! - [`stored_document`]: The [`StoredDocument`](stored_document::StoredDocument) that a listing
//!   found in place of a document file.

pub mod document_key;
pub mod document_metadata;
pub mod key_prefix;
pub mod listed_document;
pub mod stored_document;
