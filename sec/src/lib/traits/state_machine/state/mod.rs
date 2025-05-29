//! # SEC State Machine Traits
//!
//! This module defines the core traits for SEC-specific state types, as well as their associated context and state data.
//! For transition and super state traits, see the [`crate::traits::state_machine::transition`] and [`crate::traits::state_machine::super_state`] modules.
//! It extends the generic [`state_maschine`] framework with domain-specific abstractions for robust, type-safe, and testable workflows
//! in SEC data processing pipelines.
//!
//! ## Modules
//! - [`context_data`]: Traits for defining context data used within SEC state machines.
//! - [`state_data`]: Traits for defining state data used within SEC state machines.
//!
//! ## Usage
//! Implement the [`StateMachine`](super::StateMachine) trait for your SEC-specific state machine types to leverage the extensible framework
//! and integrate with concrete state, context, and data implementations found in [`crate::implementations`].
//!
//! See the documentation for each submodule for details on trait requirements and usage patterns.

use async_trait::async_trait;

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
#[async_trait]
pub trait State: SMState {
    /// Computes the output data for the SEC state.
    ///
    /// # Errors
    ///
    /// Returns an error convertible into a `StateError` if the output data computation fails.
    async fn compute_output_data_async(&mut self) -> Result<(), impl Into<StateError>>;
}
