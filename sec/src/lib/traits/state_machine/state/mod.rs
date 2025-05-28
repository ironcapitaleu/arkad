//! # SEC State Machine Traits
//!
//! This module defines the core traits for building SEC-specific state machines, states, super states, and transitions.
//! It extends the generic [`state_maschine`] framework with domain-specific abstractions for robust, type-safe, and testable workflows
//! in SEC data processing pipelines.
//!
//! ## Modules
//! - [`state`]: Traits for defining states, context data, and state data used within SEC state machines.
//! - [`super_state`]: Traits for hierarchical (composite) states, supporting advanced state machine patterns.
//!
//! ## Usage
//! Implement the [`SecStateMachine`] trait for your SEC-specific state machine types to leverage the extensible framework
//! and integrate with concrete state, context, and data implementations found in [`crate::implementations`].
//!
//! See the documentation for each submodule for details on trait requirements and usage patterns.

use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;

pub mod context_data;
pub mod state_data;

pub use context_data::ContextData;
pub use state_data::StateData;

/// Trait for SEC-specific states, extending the generic state machine state with domain error handling.
///
/// Implement this trait for SEC state types to provide custom output computation logic with error propagation.
///
/// # Errors
///
/// Returns an error convertible into a [`StateError`] if output data computation fails.
pub trait State: SMState {
    /// Computes the output data for the SEC state.
    ///
    /// # Errors
    ///
    /// Returns an error convertible into a `StateError` if the output data computation fails.
    fn compute_output_data(&mut self) -> Result<(), impl Into<StateError>>;
}
