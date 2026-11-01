//! # Test Utilities
//!
//! Shared test helpers and fixtures for the `sec` crate. They compile under `#[cfg(test)]`, and under
//! the `test-fixtures` feature for the tests of the binaries and the integration tests.
//!
//! ## Modules
//!
//! - [`fixtures`]: Reusable sample states, clients, requests, and responses.

pub mod fixtures;
