//! # Storage
//!
//! Provides the arkad workspace's persistence layer. Code reads and writes domain records through
//! the backend-agnostic traits, which return the [`error`] types. The concrete backends implement
//! those traits.
//!
//! [`ReadRepository`] reads a record by key. [`WriteRepository`] persists a record.
//! [`ReadWriteRepository`] names a store that does both. A component depends on the trait for the
//! access it needs, so a write from a read-only caller does not compile.
//!
//! The traits are expressed in domain types and name no concrete database or backend. Each backend
//! lives under [`implementations`] and depends on the traits, never the other way round.
//!
//! ## Modules
//!
//! - [`error`]: The error types the crate returns and the conversions between them.
//! - [`implementations`]: The concrete backends, one module per storage medium.
//! - [`traits`]: The traits for reading and writing records.
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
pub mod implementations;
pub mod traits;

pub use error::{BackendError, ErrorKind, ReadError, WriteError};
pub use traits::repository::{ReadRepository, ReadWriteRepository, WriteRepository};

#[cfg(test)]
pub mod tests;
