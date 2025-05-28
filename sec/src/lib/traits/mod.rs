//! # SEC State Machine Traits
//!
//! This module defines the core traits required for building state machines, states, transitions, context, and data
//! within the SEC state machine library. These traits provide the extensible foundation for implementing robust,
//! type-safe, and testable state machine workflows for SEC data processing.
//!
//! ## Modules
//! - [`state_machine`]: Core traits for state machines, states, context data, and state data.
//! - [`error`]: Traits for error handling and domain error conversions.
//!
//! ## Usage
//! These traits are implemented by concrete types in the [`crate::implementations`] and used throughout the library
//! to ensure consistent interfaces for state management, data validation, and error propagation.
//!
//! ## See Also
//! - [`crate::implementations`]: Concrete implementations of states and state machines.
//! - [`crate::shared`]: Shared domain types and utilities used by trait implementations.
//! - [`crate::error`]: Strongly-typed error handling for all state machine operations.

pub mod state_machine;

pub mod error;
