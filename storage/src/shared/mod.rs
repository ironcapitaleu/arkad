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

pub mod document_key;

pub use document_key::{DocumentKey, InvalidDocumentKey};
