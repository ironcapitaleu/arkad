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
//! - [`sha256`]: The [`sha256_hex`](sha256::sha256_hex) function that computes the digest a
//!   metadata file holds.

pub mod document_key;
pub mod document_metadata;
pub mod sha256;
