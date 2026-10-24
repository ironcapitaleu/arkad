//! # Filesystem Errors
//!
//! Provides the error types the
//! [`FilesystemRepository`](crate::implementations::repository::filesystem::FilesystemRepository)
//! raises.
//!
//! ## Modules
//!
//! - [`failed_filesystem_operation`]: The [`FailedFilesystemOperation`] for an operation on a path
//!   that did not complete.
//! - [`invalid_document_key`]: The [`InvalidDocumentKey`] for a path the store rejects.

pub mod failed_filesystem_operation;
pub mod invalid_document_key;

pub use failed_filesystem_operation::{
    FailedFilesystemOperation, FailedFilesystemOperationReason, FilesystemOperation,
};
pub use invalid_document_key::InvalidDocumentKey;
