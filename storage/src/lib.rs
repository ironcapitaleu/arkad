//! # Storage
//!
//! Provides the arkad workspace's backend-agnostic persistence interface: the [`Repository`] trait
//! that code persists domain records through, and the [`error`] types it returns.
//!
//! The interface is expressed in domain types and holds these abstractions only, naming no concrete
//! database or backend.
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
