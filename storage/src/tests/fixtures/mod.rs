//! # Common Test Fixtures
//!
//! Reusable test doubles and sample records for building `storage` tests.
//!
//! ## Modules
//!
//! - [`fake_read_repository`]: A fake [`ReadRepository`](crate::ReadRepository) initialized
//!   with key-to-record entries.
//! - [`fake_read_write_repository`]: A fake that implements both
//!   [`ReadRepository`](crate::ReadRepository) and [`WriteRepository`](crate::WriteRepository),
//!   and round-trips a record.
//! - [`fake_write_repository`]: A fake [`WriteRepository`](crate::WriteRepository) that records
//!   what it persists.
//! - [`sample_raw_document`]: A fixed [`RawDocument`](crate::implementations::repository::filesystem::RawDocument)
//!   for the filesystem backend's tests.

pub mod fake_read_repository;
pub mod fake_read_write_repository;
pub mod fake_write_repository;
pub mod sample_raw_document;
