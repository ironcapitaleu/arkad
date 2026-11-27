//! # Lease Repository Implementations
//!
//! Provides the backends that implement [`LeaseRepository`](crate::LeaseRepository).
//!
//! ## Modules
//!
//! - [`filesystem`]: The [`FilesystemLeaseRepository`] storing each lease as a file under a root
//!   directory.

pub mod filesystem;

pub use filesystem::FilesystemLeaseRepository;
