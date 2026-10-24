//! # Filesystem Errors
//!
//! Provides the error types the [`FilesystemRepository`] raises.
//!
//! An error that a type in [`shared`] raises lives beside that type.
//!
//! ## Modules
//!
//! - [`failed_filesystem_operation`]: The [`FailedFilesystemOperation`] for an operation on a path
//!   that did not complete.
//!
//! [`FilesystemRepository`]: crate::implementations::repository::filesystem::FilesystemRepository
//! [`shared`]: crate::implementations::repository::filesystem::shared

pub mod failed_filesystem_operation;

pub use failed_filesystem_operation::{
    FailedFilesystemOperation, FailedFilesystemOperationReason, FilesystemOperation,
};
