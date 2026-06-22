//! # Transition Errors
//!
//! Provides the [`Transition`] error covering the ways moving from one state to the next can fail.
//!
//! Each variant wraps a concrete sub-error carrying the source and target state names. [`Transition`]
//! is returned by implementors of the
//! [`Transition`](crate::traits::state_machine::transition::Transition) trait and wrapped by
//! [`StateMachine`](super::StateMachine) for propagation.
//!
//! ## Modules
//!
//! - [`missing_output`]: The [`MissingOutput`] error, when the source state had no computed output.
//! - [`failed_output_conversion`]: The [`FailedOutputConversion`] error, when output couldn't become the next input.
//! - [`failed_context_conversion`]: The [`FailedContextConversion`] error, when context couldn't be carried across.
//!
//! ## Usage
//!
//! ```rust
//! use sec::error::state_machine::transition::{Transition, FailedOutputConversion};
//!
//! fn perform_transition() -> Result<(), Transition> {
//!     Err(Transition::FailedOutputConversion(
//!         FailedOutputConversion::new("SourceState", "TargetState"),
//!     ))
//! }
//! ```

use thiserror::Error;

pub mod failed_context_conversion;
pub use failed_context_conversion::FailedContextConversion;
pub mod failed_output_conversion;
pub use failed_output_conversion::FailedOutputConversion;
pub mod missing_output;
pub use missing_output::MissingOutput;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// An error occurring while transitioning between states.
///
/// Groups the ways a transition can fail: the source produced no output, its output couldn't be
/// converted to the next state's input, or its context couldn't be carried across. Each case wraps
/// a concrete sub-error.
pub enum Transition {
    /// The source state had not computed its output before the transition.
    #[error("[TransitionError] A transition error occurred, Caused by: {0}")]
    MissingOutput(#[source] MissingOutput),

    /// The source state's output could not be converted into the destination state's input.
    #[error("[TransitionError] A transition error occurred, Caused by: {0}")]
    FailedOutputConversion(#[source] FailedOutputConversion),

    /// The source state's context could not be converted into the destination state's context.
    #[error("[TransitionError] A transition error occurred, Caused by: {0}")]
    FailedContextConversion(#[source] FailedContextConversion),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fmt::Debug, hash::Hash};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_transition() {
        implements_auto_traits::<Transition>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_have_implementend_send_when_using_transition() {
        implements_send::<Transition>();
    }

    #[test]
    const fn should_implement_sync_when_using_transition() {
        implements_sync::<Transition>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_transition() {
        implements_send::<Transition>();
        implements_sync::<Transition>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_when_using_transition() {
        implements_sized::<Transition>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_transition() {
        implements_hash::<Transition>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_when_using_transition() {
        implements_partial_eq::<Transition>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_transition() {
        implements_eq::<Transition>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_when_using_transition() {
        implements_partial_ord::<Transition>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_transition() {
        implements_ord::<Transition>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_transition() {
        implements_debug::<Transition>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_transition() {
        implements_clone::<Transition>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_when_using_transition() {
        implements_unpin::<Transition>();
    }

    #[test]
    fn should_be_able_to_create_transition_failedcontextconversion_error_when_using_enum_directly()
    {
        let _result =
            Transition::FailedContextConversion(FailedContextConversion::new("StateA", "StateB"));
    }
}
