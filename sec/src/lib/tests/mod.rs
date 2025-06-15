//! # Test Utilities Module
//!
//! This module contains shared utilities, fixtures, and common code for use across the `sec`
//! crate's test suite. It is conditionally compiled and is only available when running tests,
//! as enabled by the `#[cfg(test)]` attribute on its declaration in `lib.rs`.
//!
//! ## Purpose
//! The primary goal of this module is to provide a centralized location for test helpers,
//! reducing code duplication and improving the organization of the test suite. It includes
//! common state machine fixtures and other utilities that can be reused in unit, integration,
//! and documentation tests.
//!
//! ## Modules
//! - [`common`]: Contains common test fixtures, such as sample state implementations.

pub mod common;
