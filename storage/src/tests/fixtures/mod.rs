//! # Common Test Fixtures
//!
//! Reusable test doubles for building `storage` tests without a real backend.
//!
//! ## Modules
//!
//! - [`fake_repository`]: A fake [`Repository`](crate::Repository) that records what it persists.

pub mod fake_repository;
