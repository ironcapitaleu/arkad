//! # Repository Implementations
//!
//! Provides the backends that implement [`ReadRepository`](crate::ReadRepository) and
//! [`WriteRepository`](crate::WriteRepository).
//!
//! ## Modules
//!
//! - [`filesystem`]: The [`FilesystemRepository`](filesystem::FilesystemRepository) storing
//!   documents as files under a root directory.

pub mod filesystem;
