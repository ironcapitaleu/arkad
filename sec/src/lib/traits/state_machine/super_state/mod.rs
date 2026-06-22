//! # Super-State Trait
//!
//! Provides the [`SuperState`] trait for hierarchical states: a state that is itself a state
//! machine.
//!
//! This is what lets a phase like extraction appear as a single state to the outer pipeline while
//! internally driving its own sub-states. The extract and transform super-states are the concrete
//! implementors.

use crate::prelude::{State, StateMachine};

/// A state that is itself a state machine over sub-states.
///
/// Bundles the [`State`] and [`StateMachine`] bounds so a super-state can both participate as one
/// state in a parent machine and drive its own inner states, restricting those inner states to the
/// SEC [`State`] trait.
///
/// # Type Parameters
///
/// - `S`: The active inner state type. Must implement [`State`].
pub trait SuperState<S>: State + StateMachine<S>
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
        tests::fixtures::{
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

        let result = super_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_inner_state_name_when_accessing_current_state() {
        let super_state = SampleSecSuperState::<SampleSecState>::new();

        let expected_result = "Sample SEC State";

        let result = super_state.current_state().state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_compute_output_for_inner_state_when_calling_compute_on_super_state() {
        let mut super_state = SampleSecSuperState::<SampleSecState>::new();

        // The super state's output is not computed, but the inner state's is.
        assert!(super_state.output_data().is_none());
        assert!(super_state.current_state().output_data().is_none());

        super_state
            .compute_output_data_async()
            .await
            .expect("Valid test state should compute output successfully");

        assert!(super_state.output_data().is_none()); // Super state output is still None
        assert!(super_state.current_state().output_data().is_some()); // Inner state has output
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_super_state() {
        let super_state = SampleSecSuperState::<SampleSecState>::new();
        let expected_result = &SampleSecSuperStateContext::default();
        let result = super_state.context_data();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_input_data_when_in_initial_super_state() {
        let super_state = SampleSecSuperState::<SampleSecState>::new();
        let expected_result = &SampleSecSuperStateData::default();
        let result = super_state.input_data();
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
