//! # Storage — persistence ports
//!
//! Backend-agnostic persistence ports for the arkad workspace. The crate owns the *word*
//! "storage" so no trait is named `Storage`; callers depend on these ports and never on a concrete
//! database, so swapping the physical backend (Postgres, Iceberg, a graph DB, or any mixture) is a
//! new implementation behind the same interface rather than a rewrite of the code that depends on
//! it.
//!
//! This is the first, deliberately minimal slice: the neutral [`Repository`] port and the
//! write-side [`error::ErrorKind`] hierarchy. The tier capability traits (`RawStore` /
//! `GraphStore` / `FactStore`), a composing repository, the read surface, and every concrete
//! backend are layered on in their own follow-up slices so this foundation stays small and
//! reviewable.
//!
//! ## Modules
//!
//! - [`error`]: The write-side error hierarchy topped by [`ErrorKind`], mirroring the `sec` crate's
//!   module-per-level, `From` upcast / `TryFrom` downcast shape.
//! - [`repository`]: The neutral [`Repository`] port — `type Record` + `persist`, backend-blind.
//!
//! ## Usage
//!
//! ```rust
//! use storage::{ErrorKind, WriteError};
//!
//! let _err = ErrorKind::Write(WriteError::ConflictingWrite {
//!     reason: "duplicate accession number".to_string(),
//! });
//! ```

pub mod error;
pub mod repository;

pub use error::{BackendError, ErrorKind, WriteError};
pub use repository::Repository;

#[cfg(test)]
pub mod tests;
