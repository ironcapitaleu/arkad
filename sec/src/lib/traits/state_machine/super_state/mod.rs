//! # `SuperState` Trait
//!
//! This module defines the [`SuperState`] trait for hierarchical (composite) states in the SEC state machine framework.
//! A `SuperState` is a state that itself acts as a state machine, enabling advanced patterns such as nested workflows,
//! composite states, and encapsulated sub-state machines within a parent state machine.
//!
//! ## Overview
//! The [`SuperState`] trait extends the generic [`state_maschine::prelude::SuperState`] trait, adding SEC-specific
//! constraints and integration with the [`State`] and [`super::StateMachine`] traits from this crate. This allows for robust, type-safe, and testable
//! hierarchical state machines in SEC data processing pipelines.
//!
//! ## Usage
//! Implement this trait for any state that should also be able to function as its own state machine, managing its own internal states
//! while participating as a single state in a parent state machine.
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::State`]: Trait for defining individual states.
//! - [`crate::traits::state_machine::StateMachine`]: Trait for SEC-specific state machines.
//! - [`state_maschine::state_machine::super_state::SuperState`]: Underlying framework trait for super states.
//!

use state_maschine::prelude::SuperState as SMSuperState;

use crate::traits::state_machine::state::State;

/// The `SuperState` trait is used for hierarchical (composite) states in the SEC state machine framework.
///
/// This trait extends the generic [`state_maschine::state_machine::super_state::SuperState`] trait, but restricts the state type `S`
/// to types that implement the SEC-specific [`State`] trait. This ensures that all sub-states within a `SuperState`
/// conform to the SEC state machine's requirements.
///
/// # Type Parameters
/// - `S`: The state type, which must implement [`State`].
pub trait SuperState<S>: SMSuperState<S>
where
    S: State,
{
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use tokio;

    use crate::{
        prelude::*,
        tests::common::{
            sample_sec_state::SampleSecState,
            sample_sec_super_state::{
                SampleSecSuperState, SampleSecSuperStateContext, SampleSecSuperStateData,
            },
        },
    };

    #[test]
    fn should_return_super_state_name_when_in_sample_super_state() {
        let super_state = SampleSecSuperState::<SampleSecState>::new();

        let expected_result = "Sample SEC SuperState (Current: Sample SEC State)";

        let result = super_state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_inner_state_name_when_accessing_current_state() {
        let super_state = SampleSecSuperState::<SampleSecState>::new();

        let expected_result = "Sample SEC State";

        let result = super_state.get_current_state().get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_compute_output_for_inner_state_when_calling_compute_on_super_state() {
        let mut super_state = SampleSecSuperState::<SampleSecState>::new();

        // The super state's output is not computed, but the inner state's is.
        assert!(super_state.get_output_data().is_none());
        assert!(super_state.get_current_state().get_output_data().is_none());

        super_state
            .compute_output_data_async()
            .await
            .expect("Computation should succeed");

        assert!(super_state.get_output_data().is_none()); // Super state output is still None
        assert!(super_state.get_current_state().get_output_data().is_some()); // Inner state has output
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_super_state() {
        let super_state = SampleSecSuperState::<SampleSecState>::new();
        let expected_result = &SampleSecSuperStateContext::default();
        let result = super_state.get_context_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_input_data_when_in_initial_super_state() {
        let super_state = SampleSecSuperState::<SampleSecState>::new();
        let expected_result = &SampleSecSuperStateData::default();
        let result = super_state.get_input_data();
        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_super_state_trait() {
        implements_auto_traits::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_send<T: Send>() {}
    #[test]
    const fn should_implement_send_when_implementing_super_state_trait() {
        implements_send::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_sync<T: Sync>() {}
    #[test]
    const fn should_implement_sync_when_implementing_super_state_trait() {
        implements_sync::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_super_state_trait() {
        implements_sized::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_super_state_trait() {
        implements_hash::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_super_state_trait() {
        implements_partial_eq::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_super_state_trait() {
        implements_eq::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_super_state_trait() {
        implements_partial_ord::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_super_state_trait() {
        implements_ord::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_super_state_trait() {
        implements_debug::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_super_state_trait() {
        implements_clone::<SampleSecSuperState<SampleSecState>>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_super_state_trait() {
        implements_unpin::<SampleSecSuperState<SampleSecState>>();
    }
}
