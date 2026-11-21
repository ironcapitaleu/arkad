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
//! - [`silver_object_store`]: The [`SilverObject`](silver_object_store::SilverObject) and the
//!   [`FilesystemSilverObjectStore`](silver_object_store::FilesystemSilverObjectStore) storing
//!   each silver object once as a file under a root directory.

pub mod filesystem;
pub mod silver_object_store;
