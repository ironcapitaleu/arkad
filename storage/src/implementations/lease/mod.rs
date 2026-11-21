//! # Lease Implementations
//!
//! Provides the backends that implement [`LeaseStore`](crate::LeaseStore).
//!
//! ## Modules
//!
//! - [`filesystem`]: The [`FilesystemLeaseStore`](filesystem::FilesystemLeaseStore) storing each
//!   lease as a file under a root directory.

pub mod filesystem;
