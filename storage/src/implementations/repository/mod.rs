//! # Repository Implementations
//!
//! Provides the backends that implement [`ReadRepository`](crate::ReadRepository) and
//! [`WriteRepository`](crate::WriteRepository).
//!
//! ## Modules
//!
//! - [`filesystem`]: [`FilesystemRepository`](filesystem::FilesystemRepository), which stores
//!   documents as files under a root directory.

pub mod filesystem;
