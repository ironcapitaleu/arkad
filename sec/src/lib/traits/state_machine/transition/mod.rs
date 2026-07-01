//! # Transition Trait
//!
//! Provides the [`Transition`] trait modeling a move from one SEC state to another.
//!
//! It refines the generic [`state_maschine`] transition with an SEC-specific method that reports
//! rich [`TransitionError`]s instead of the framework's static strings.

use state_maschine::prelude::Transition as SMTransition;

use crate::error::state_machine::transition::Transition as TransitionError;
use crate::traits::state_machine::state::State;

/// A transition from source state `T` to target state `U`.
///
/// Refines the generic [`SMTransition`], constraining both ends to the SEC [`State`] trait and
/// adding [`transition_to_next_state_sec`](Transition::transition_to_next_state_sec) for
/// domain-typed error reporting. Implemented by the extract and transform super-states for each
/// valid edge.
///
/// # Type Parameters
///
/// - `T`: The source state type. Must implement [`State`].
/// - `U`: The target state type. Must implement [`State`].
pub trait Transition<T, U>: SMTransition<T, U>
where
    T: State,
    U: State,
{
    /// Transitions to the next state, reporting failures as SEC [`TransitionError`]s.
    ///
    /// Mirrors the generic `transition_to_next_state` but returns a domain-typed error rather than
    /// a static string, enabling richer diagnostics.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError`] if the move fails, e.g.
    /// [`FailedOutputConversion`](TransitionError::FailedOutputConversion) when the source's output
    /// can't become the target's input, or
    /// [`FailedContextConversion`](TransitionError::FailedContextConversion) when its context can't
    /// be carried across.
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError>;
}
