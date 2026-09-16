//! # Storage
//!
//! Provides the arkad workspace's backend-agnostic persistence interface: the traits that code
//! reads and writes domain records through, and the [`error`] types they return.
//!
//! [`ReadRepository`] reads a record by key. [`WriteRepository`] persists a record.
//! [`ReadWriteRepository`] names a store that does both. A component depends on the trait for the
//! access it needs, so a write from a read-only caller does not compile.
//!
//! The interface is expressed in domain types and holds these abstractions only, naming no concrete
//! database or backend.
//!
//! ## Modules
//!
//! - [`error`]: The error types the crate returns and the conversions between them.
//! - [`repository`]: The traits for reading and writing records.
//!
//! ## Usage
//!
//! A component names the trait it needs. This one reads, so it has no way to write:
//!
//! ```rust
//! use storage::{ReadError, ReadRepository};
//!
//! async fn load<R: ReadRepository>(
//!     reader: &R,
//!     key: R::Key,
//! ) -> Result<Option<R::Record>, ReadError> {
//!     reader.get(key).await
//! }
//! ```

pub mod error;
pub mod repository;

pub use error::{BackendError, ErrorKind, ReadError, WriteError};
pub use repository::{ReadRepository, ReadWriteRepository, WriteRepository};

#[cfg(test)]
pub mod tests;
