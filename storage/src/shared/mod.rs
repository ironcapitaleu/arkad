//! # Shared
//!
//! Provides the domain types every storage backend uses.
//!
//! A backend stores documents, and every backend names them the same way. These types live outside
//! [`implementations`](crate::implementations), so a second backend reuses them without depending
//! on the first.
//!
//! ## Modules
//!
//! - [`document_key`]: The [`DocumentKey`](document_key::DocumentKey) naming one document, and the
//!   [`InvalidDocumentKey`](document_key::InvalidDocumentKey) for a rejected path.
//! - [`input_document`]: The [`InputDocument`](input_document::InputDocument) pointing at a
//!   document that a stored document came from.
//! - [`producer`]: The [`Producer`](producer::Producer) naming the code that wrote a document.

pub mod document_key;
pub mod input_document;
pub mod producer;
