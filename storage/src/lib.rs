//! # Storage — persistence ports
//!
//! Backend-agnostic persistence ports for the arkad workspace. The crate owns the *word*
//! "storage" so no trait is named `Storage`; callers depend on these ports and never on a concrete
//! database, so swapping the physical backend (Postgres, Iceberg, a graph DB, or any mixture) is a
//! new implementation behind the same interface rather than a rewrite of the code that depends on
//! it.
//!
//! This crate holds the persistence *ports* only: the neutral [`Repository`] port and the
//! write-side [`error::ErrorKind`] hierarchy. Concrete backends live in separate crates, so
//! depending on these ports never pulls in a backend.
//!
//! ## Modules
//!
//! - [`error`]: The write-side error hierarchy topped by [`ErrorKind`] — module-per-level, with
//!   `From` upcast / `TryFrom` downcast between levels.
//! - [`repository`]: The neutral [`Repository`] port — `type Record` + `persist`, backend-blind.
//!
//! ## Usage
//!
//! ```rust
//! use storage::{ErrorKind, WriteError};
//!
//! let _err = ErrorKind::Write(WriteError::conflicting_write("duplicate accession number"));
//! ```

pub mod error;
pub mod repository;

pub use error::{BackendError, ErrorKind, WriteError};
pub use repository::Repository;

#[cfg(test)]
pub mod tests;
