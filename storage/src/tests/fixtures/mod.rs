//! # Common Test Fixtures
//!
//! Reusable test doubles for building `storage` tests without a real backend.
//!
//! ## Modules
//!
//! - [`fake_read_repository`]: A fake [`ReadRepository`](crate::ReadRepository) seeded with
//!   key-to-record entries.
//! - [`fake_store`]: A fake that implements both ports and round-trips a record.
//! - [`fake_write_repository`]: A fake [`WriteRepository`](crate::WriteRepository) that records
//!   what it persists.

pub mod fake_read_repository;
pub mod fake_store;
pub mod fake_write_repository;
