//! # Repository
//!
//! Provides the persistence ports, split by access capability.
//!
//! Each port names one capability. A caller depends on the port for the access it needs, so a
//! read-only caller has no write method in scope.
//!
//! ## Modules
//!
//! - [`write_repository`]: [`WriteRepository`] — persists a record.
//!
//! ## Usage
//!
//! ```rust
//! use storage::WriteRepository;
//!
//! fn write_unit<W: WriteRepository>() -> Option<W::Record> {
//!     None
//! }
//! ```

pub mod write_repository;

pub use write_repository::WriteRepository;
