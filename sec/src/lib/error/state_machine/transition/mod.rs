//! # Transition Error Types
//!
//! This module defines the [`Transition`] error enum, representing errors that can occur during state transitions
//! within a state machine. These errors are used throughout the SEC state machine framework to provide
//! strongly-typed, descriptive error handling for transition operations.
//!
//! ## Overview
//!
//! The [`Transition`] enum covers two main failure scenarios:
//! - [`FailedOutputConversion`](Transition::FailedOutputConversion): Occurs when the output data of the source state cannot be converted into the input data of the destination state.
//! - [`FailedContextConversion`](Transition::FailedContextConversion): Occurs when the context data of the source state cannot be converted into the context data of the destination state.
//!
//! These errors are intended to be used by implementers of the [`Transition`](crate::traits::state_machine::transition::Transition) trait
//! and are surfaced by the state machine error handling system (see [`crate::error`] and [`crate::error::state_machine`]).
//!
//! ## Usage
//!
//! These error types are typically returned by transition logic in the state machine implementation, and can be
//! matched against to provide detailed diagnostics or recovery strategies.
//!
//! ## Related Modules
//! - [`crate::error`]: Top-level error types for the SEC state machine library.
//! - [`crate::error::state_machine`]: Error types specific to state machine operations.
//! - [`crate::traits::state_machine::transition`]: The trait defining transition behavior in the state machine framework.
//!
//! ## Example
//!
//! ```rust
//! use sec::error::state_machine::transition::Transition;
//!
//! fn perform_transition() -> Result<(), Transition> {
//!     // ... transition logic ...
//!     Err(Transition::FailedOutputConversion)
//! }
//! ```

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Represents errors that can occur during state transitions within the state machine framework.
///
/// This enum is used to signal failure scenarios when, for example, converting state output or context data
/// between states. It is intended for use by implementers of the [`Transition`](crate::traits::state_machine::transition::Transition) trait,
/// and is surfaced by the state machine error handling system.
///
/// See the module-level documentation for more details and usage examples.
pub enum Transition {
    /// Failed to convert output of the source state into the input of the destination state.
    ///
    /// This error variant indicates that the output data produced by the source state could not
    /// be transformed or mapped into the input data required by the destination state during a transition.
    FailedOutputConversion,

    /// Failed to convert context of the source state into the context of the destination state.
    ///
    /// This error variant indicates that the context data associated with the source state could not
    /// be transformed or mapped into the context required by the destination state during a transition.
    FailedContextConversion,
}

impl std::fmt::Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem occured during transition operations.")
    }
}

impl std::error::Error for Transition {}

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
        let _result = Transition::FailedContextConversion;
    }
}
