//! # Common Test Fixtures
//!
//! This module groups common fixtures and helpers used throughout the test suite.
//! It provides reusable components, such as sample state machine and state implementations,
//! that can be used to build consistent and maintainable tests.
//!
//! ## Modules
//! - [`fake_http_client`]: Provides fake HTTP client implementations for testing.
//! - [`sample_sec_state`]: Provides a generic "Hello World"! implementation of a `State`
//!   for testing and reimplementation purposes.
//! - [`sample_sec_super_state`]: Provides a simple super state implementation that makes use
//! of [`sample_sec_state`] for testing and reimplementation purposes.
//!
pub mod fake_http_client;
pub mod sample_sec_state;
pub mod sample_sec_super_state;
