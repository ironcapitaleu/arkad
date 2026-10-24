//! # Filesystem Errors
//!
//! Provides the error types the
//! [`FilesystemRepository`](crate::implementations::repository::filesystem::FilesystemRepository)
//! raises.
//!
//! ## Modules
//!
//! - [`filesystem_error`]: The [`FilesystemError`] for a failed operation on a path.
//! - [`invalid_document_key`]: The [`InvalidDocumentKey`] for a path the store rejects.

pub mod filesystem_error;
pub mod invalid_document_key;

pub use filesystem_error::{FilesystemError, FilesystemErrorReason, FilesystemOperation};
pub use invalid_document_key::InvalidDocumentKey;
