//! # Repository
//!
//! Provides the traits through which code reads and writes records.
//!
//! [`ReadRepository`] gives a caller `get` and nothing else. [`WriteRepository`] gives it
//! `persist` and nothing else. A caller depends on the trait for the access it needs, so a
//! read-only caller has no write method in scope.
//!
//! ## Modules
//!
//! - [`read_repository`]: [`ReadRepository`] — reads a record by key.
//! - [`read_write_repository`]: [`ReadWriteRepository`] — names a store that reads and writes.
//! - [`write_repository`]: [`WriteRepository`] — persists a record.
//!
//! ## Usage
//!
//! ```rust
//! use storage::{ReadRepository, WriteRepository};
//!
//! fn takes_a_store<S>(_store: S)
//! where
//!     S: ReadRepository<Record = String, Key = String> + WriteRepository<Record = String>,
//! {
//! }
//! ```

pub mod read_repository;
pub mod read_write_repository;
pub mod write_repository;

pub use read_repository::ReadRepository;
pub use read_write_repository::ReadWriteRepository;
pub use write_repository::WriteRepository;
