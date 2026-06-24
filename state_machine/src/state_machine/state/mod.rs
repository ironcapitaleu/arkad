//! # State Trait
//!
//! Provides the [`State`] trait, one node in a state machine, along with its [`Context`] and
//! [`StateData`] traits.
//!
//! ## Modules
//!
//! - [`context`]: The [`Context`] trait for a state's ambient context.
//! - [`state_data`]: The [`StateData`] trait for a state's input and output data.

use std::{fmt::Debug, hash::Hash};

pub mod context;
pub mod state_data;

pub use context::Context;
pub use state_data::StateData;

/// One node in a state machine: holds input, context, and computed output.
///
/// A state reads typed input and context and computes its output on demand. The supertrait bounds
/// keep every state thread-safe, comparable, and hashable so machines can store and reason about
/// them uniformly.
///
/// # Associated Types
///
/// - `InputData`: The data the state processes. Must implement [`StateData`].
/// - `OutputData`: The data the state produces. Must implement [`StateData`].
/// - `Context`: The state's ambient context. Must implement [`Context`].
///
/// # Required Traits
///
/// Implementors must be `Debug + Send + Sync + Unpin + Clone + PartialEq + PartialOrd + Hash + Eq +
/// Ord` so states are thread-safe and usable as keys in ordered or hashed collections.
pub trait State:
    Debug + Send + Sync + Unpin + Clone + PartialEq + PartialOrd + Hash + Eq + Ord
{
    /// The data the state processes.
    type InputData: StateData;
    /// The data the state produces.
    type OutputData: StateData;
    /// The state's ambient context.
    type Context: Context;

    /// Returns the state's name, for identification in logs and diagnostics.
    fn state_name(&self) -> impl ToString;

    /// Returns a reference to the state's input data.
    fn input_data(&self) -> &Self::InputData;

    /// Computes the output data from the input and stores it on the state.
    fn compute_output_data(&mut self);

    /// Returns the computed output data, or `None` if it has not been computed yet.
    fn output_data(&self) -> Option<&Self::OutputData>;

    /// Returns `true` if the output data has been computed.
    ///
    /// Defaults to checking whether [`output_data`](State::output_data) is `Some`; override for
    /// more involved checks.
    fn has_output_data_been_computed(&self) -> bool {
        self.output_data().is_some()
    }

    /// Returns a reference to the state's context data.
    fn context_data(&self) -> &Self::Context;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::{SampleState, SampleStateContext, SampleStateData};
    use pretty_assertions::assert_eq;

    #[test]
    fn should_return_name_of_sample_state_when_in_sample_state() {
        let sample_state = SampleState::default();

        let expected_result = String::from("Sample State");

        let result = sample_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_struct_as_input_data_when_output_data_has_not_been_computed_in_state()
     {
        let sample_state = SampleState::default();

        let expected_result = &SampleStateData::default();

        let result = sample_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_struct_as_input_data_when_in_initial_sample_state() {
        let sample_state = SampleState::default();

        let expected_result = &SampleStateData::default();

        let result = sample_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let sample_state = SampleState::default();

        let _result = sample_state
            .output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let sample_state = SampleState::default();

        let expected_result = false;

        let result = sample_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_true_when_state_has_computed_the_output() {
        let mut sample_state = SampleState::default();

        let expected_result = true;

        sample_state.compute_output_data();
        let result = sample_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let sample_state = SampleState::default();

        let expected_result = &SampleStateContext::default();

        let result = sample_state.context_data();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<SampleState>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<SampleState>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<SampleState>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<SampleState>();
        implements_sync::<SampleState>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<SampleState>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<SampleState>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<SampleState>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<SampleState>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<SampleState>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<SampleState>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_trait() {
        implements_default::<SampleState>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<SampleState>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<SampleState>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<SampleState>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let sample_state = &SampleState::default();
        let ref_to_sample_state = &SampleState::default();

        let expected_result = sample_state.context_data();

        let result = ref_to_sample_state.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_true_when_reference_state_has_computed_the_output() {
        let ref_to_sample_state = &mut SampleState::default();

        let expected_result = true;

        ref_to_sample_state.compute_output_data();
        let result = ref_to_sample_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_sample_state = &mut SampleState::default();

        let expected_result = false;

        let result = ref_to_sample_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_sample_state = &SampleState::default();

        let _result = ref_to_sample_state
            .output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_name_of_sample_state_when_calling_reference_to_sample_state() {
        let ref_to_sample_state = &SampleState::default();

        let expected_result = String::from("Sample State");

        let result = ref_to_sample_state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_output_data_has_not_been_computed_in_reference_state()
     {
        let ref_to_sample_state = &SampleState::default();

        let expected_result = &SampleStateData::default();

        let result = ref_to_sample_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_sample_state_in_initial_state()
    {
        let ref_to_sample_state = &SampleState::default();

        let expected_result = &SampleStateData::default();

        let result = ref_to_sample_state.input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_change_input_data_when_computing_output_data() {
        let mut sample_state = SampleState::default();

        let expected_result = &sample_state.input_data().clone();

        sample_state.compute_output_data();
        let result = sample_state.input_data();

        assert_eq!(result, expected_result);
    }
}
