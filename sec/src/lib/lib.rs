//! # SEC State Machine Library
//!
//! This crate provides a modular and extensible framework for processing SEC filings using state machines in Rust.
//! It is designed for robust, type-safe, and testable workflows, supporting hierarchical state machines, error handling, and flexible state/context/data management.
//!
//! ## Modules
//! - [`prelude`]: Convenient re-exports of core traits for easy use in downstream crates.
//! - [`traits`]: Core state machine, state, context, and data traits for extensibility and integration.
//! - [`implementations`]: Concrete state machine and state implementations for SEC data extraction, transformation, and loading.
//! - [`shared`]: Shared utilities and domain types (e.g., CIK parsing) used across the library.
//! - [`error`]: Strongly-typed error handling for state machines, states, and transitions, with comprehensive error kinds and conversions.
//!
//! ## Usage
//! Import the [`prelude`] module for quick access to the main traits and types. See the crate-level documentation and individual modules for detailed usage examples and design diagrams.
//!
//! ## License
//! Licensed under MIT or Apache-2.0. See LICENSE-* files for details.

pub mod prelude;

pub mod traits;

pub mod implementations;

pub mod shared;

pub mod error;

#[cfg(test)]
pub mod tests;
