//! # Storage — persistence ports
//!
//! Provides the backend-agnostic persistence ports for the arkad workspace: the abstract interfaces
//! code persists through, in domain types, without naming a concrete database. The crate owns the
//! word "storage", so no trait is named `Storage`.
//!
//! It contains the ports only — the [`Repository`] port and the [`error`] types it returns — and no
//! concrete backend.
//!
//! ## Modules
//!
//! - [`error`]: The error types the crate returns and the conversions between them.
//! - [`repository`]: The [`Repository`] persistence port.
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
