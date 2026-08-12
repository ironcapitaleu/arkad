//! # Storage
//!
//! Provides the arkad workspace's backend-agnostic persistence interface: the abstract
//! [`Repository`] trait code persists through — in domain types, without naming a concrete database
//! — and the [`error`] types it returns. The crate owns the word "storage", so no trait is named
//! `Storage`.
//!
//! It holds these abstractions only, and no concrete backend.
//!
//! ## Modules
//!
//! - [`error`]: The error types the crate returns and the conversions between them.
//! - [`repository`]: The [`Repository`] trait for persisting records.
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
