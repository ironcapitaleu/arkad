//! # Repository Implementations
//!
//! Provides the backends that implement [`ReadRepository`](crate::ReadRepository),
//! [`WriteRepository`](crate::WriteRepository), or both. A backend that implements both is also a
//! [`ReadWriteRepository`](crate::ReadWriteRepository).
//!
//! ## Modules
//!
//! - [`filesystem`]: The [`FilesystemRepository`](filesystem::FilesystemRepository) storing
//!   documents as files under a root directory.
//! - [`write_once`]: The [`Blob`](write_once::Blob) and the
//!   [`WriteOnceFilesystemRepository`](write_once::WriteOnceFilesystemRepository) storing
//!   each blob once as a file under a root directory.

pub mod filesystem;
pub mod write_once;
