//! # SEC State Machine `Transition` Trait
//!
//! This module defines the [`Transition`] trait for SEC-specific state machines, extending the generic
//! [`state_maschine::state_machine::transition::Transition`] trait. Transitions model the movement between two states
//! (`T` and `U`) in a state machine, enabling robust, type-safe, and testable workflows for SEC data processing.
//!
//! ## Usage
//! Implement the [`Transition`] trait for types that represent valid transitions between SEC state types.
//! This trait ensures compatibility with the core state machine framework and enforces trait bounds for
//! both source (`T`) and target (`U`) states, effectively allowing the SEC state machine to transiton from one SEC state to another.
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::State`]: The trait for SEC state types.
//! - [`state_maschine::state_machine::transition::Transition`]: The generic transition trait from the underlying state machine framework.
//! - [`crate::traits::state_machine::StateMachine`]: The main trait for SEC state machines.
//!

use state_maschine::prelude::Transition as SMTransition;

use crate::error::state_machine::transition::Transition as TransitionError;
use crate::traits::state_machine::state::State;

/// The `Transition` trait defines a transition between two SEC state types within a state machine.
///
/// This trait extends the generic [`state_maschine::state_machine::transition::Transition`] trait, enforcing that both
/// the source (`T`) and target (`U`) types implement the SEC [`State`] trait. Implement this trait
/// for types that represent valid transition functions in your SEC state machine.
///
/// # Type Parameters
/// - `T`: The source state type, must implement [`State`].
/// - `U`: The target state type, must implement [`State`].
pub trait Transition<T, U>: SMTransition<T, U>
where
    T: State,
    U: State,
{
    /// Transitions the state machine to the next state using SEC-specific error handling.
    ///
    /// This method provides the same functionality as the generic `transition_to_next_state` method,
    /// but returns SEC-specific [`TransitionError`] types instead of static strings, enabling
    /// richer error handling and diagnostics.
    ///
    /// # Returns
    ///
    /// - `Ok(NewStateMachine)`: The updated state machine in the new state.
    /// - `Err(TransitionError)`: A SEC-specific error describing the failure reason.
    ///
    /// # Errors
    ///
    /// Returns a [`TransitionError`] if the transition fails, such as:
    /// - [`TransitionError::FailedOutputConversion`]: When the source state's output cannot be converted to the target state's input.
    /// - [`TransitionError::FailedContextConversion`]: When the source state's context cannot be converted to the target state's context.
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError>;
}
