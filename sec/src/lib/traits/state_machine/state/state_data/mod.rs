//! # SEC State Data Trait
//!
//! This module defines the [`StateData`] trait for SEC-specific state machines, extending the generic
//! [`state_maschine::state_machine::state::StateData`] trait with domain-aware error handling.
//!
//! State data represents the internal, mutable data associated with a state in the SEC state machine framework.
//! Implementations of this trait are responsible for encapsulating and updating the input/output data
//! for each state, supporting robust, type-safe, and testable workflows.
//!
//! ## Usage
//! Implement [`StateData`] for your SEC state data types to enable controlled updates and error propagation
//! during state transitions. The trait enforces that all updates return a strongly-typed [`crate::error::State`]
//! error on failure, ensuring consistent error handling across the state machine.
//!
//! See also:
//! - [`crate::traits::state_machine::state::ContextData`]: For context data management.
//! - [`crate::implementations`]: For concrete state data implementations used in SEC ETL pipelines.
//! - [`crate::error`]: For error types used in update operations.

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;

/// Trait for SEC-specific state data, extending the generic state machine state data trait with domain error handling.
///
/// Implement this trait for SEC state data types to provide custom update logic with error propagation.
///
/// # Errors
///
/// Returns a [`crate::error::State`] if the update fails.
pub trait StateData: SMStateData {
    /// Updates the state with new data given in the `updates` parameter.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::error::State`] if the update fails.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError>;
}
