//! # Storage
//!
//! Provides the arkad workspace's backend-agnostic persistence interface: the ports that code
//! reads and writes domain records through, and the [`error`] types they return.
//!
//! The ports are split by access capability. [`ReadRepository`] reads a record by key.
//! [`WriteRepository`] persists a record. [`ReadWriteRepository`] names a store that does both. A
//! component depends on the port for the access it needs, so a write from a read-only caller does
//! not compile.
//!
//! The interface is expressed in domain types and holds these abstractions only, naming no concrete
//! database or backend.
//!
//! ## Modules
//!
//! - [`error`]: The error types the crate returns and the conversions between them.
//! - [`repository`]: The persistence ports, split by access capability.
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
pub use repository::{ReadRepository, ReadWriteRepository, WriteRepository};

#[cfg(test)]
pub mod tests;
