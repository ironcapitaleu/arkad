//! # Test Utilities
//!
//! Shared test-only helpers and fixtures for the `xbrl` crate. Unit tests compile them under
//! `#[cfg(test)]`. Integration tests compile them through the `test-fixtures` feature.
//!
//! ## Modules
//!
//! - [`fixtures`]: Reusable test doubles and data for building `xbrl` tests.

pub mod fixtures;
