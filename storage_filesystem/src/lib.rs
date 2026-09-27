//! # Storage Filesystem
//!
//! Provides [`FileSystemRepository`], the adapter that implements the `storage` crate's persistence
//! traits against a local directory.
//!
//! One root directory holds one collection of documents, the way a table does. A [`DocumentKey`] is
//! a document's path under that root, so the key and the location are the same thing. Each document
//! is stored exactly as its source returned it, with a [`DocumentMetadata`] file beside it
//! describing the fetch.
//!
//! ## Modules
//!
//! - [`document_key`]: The [`DocumentKey`] naming one document, and the error for a rejected path.
//! - [`document_metadata`]: The [`DocumentMetadata`] stored beside a document.
//! - [`raw_document`]: The [`RawDocument`] a read returns and a write accepts.
//! - [`file_system_repository`]: The [`FileSystemRepository`] adapter.
//!
//! ## Usage
//!
//! ```rust
//! use storage_filesystem::FileSystemRepository;
//!
//! let repository = FileSystemRepository::new(std::env::temp_dir())
//!     .expect("Given the system temporary directory, opening the store should always succeed");
//!
//! let expected_result = true;
//!
//! let result = repository.root().is_dir();
//!
//! assert_eq!(result, expected_result);
//! ```

pub mod document_key;
pub mod document_metadata;
pub mod file_system_repository;
pub mod raw_document;

pub use document_key::{DocumentKey, InvalidDocumentKey};
pub use document_metadata::DocumentMetadata;
pub use file_system_repository::FileSystemRepository;
pub use raw_document::RawDocument;
