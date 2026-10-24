//! # Document Metadata
//!
//! Provides the parts of the metadata that every stored document carries.
//!
//! ## Modules
//!
//! - [`input_document`]: The [`InputDocument`] pointing at a document that another document came
//!   from.
//! - [`producer`]: The [`Producer`] naming the code that wrote a document.

pub mod input_document;
pub mod producer;

pub use input_document::InputDocument;
pub use producer::Producer;
