use std::{fmt::Debug, hash::Hash};

pub mod context;
pub mod state_data;

pub use context::Context;
pub use state_data::StateData;

/// The `State` trait defines the behavior and characteristics of a state within a state machine.
///
/// This trait encompasses the key functionalities that a state must implement, including methods for
/// retrieving state names, handling input and output data, and managing context data. It also defines
/// the data types that are associated with the state, ensuring that all states conform to a standard
/// interface for interacting with the rest of the state machine.
///
/// # Associated Types
///
/// - `InputData`: Represents the type of data input that the state processes. Must implement the `StateData` trait.
/// - `OutputData`: Represents the type of data output that the state produces. Must implement the `StateData` trait.
/// - `Context`: Represents the context or environment data associated with the state. Must implement the `Context` trait.
///
/// # Required Traits
///
/// Implementations of the `State` trait must also implement several Rust standard traits to ensure
/// thread safety, comparison, and debugging capabilities:
/// - `Debug`: Allows the state to be formatted using the `{:?}` formatter, which is useful for debugging.
/// - `Send`, `Sync`, `Unpin`: Ensure that the state can be safely transferred and accessed across threads.
/// - `Clone`, `PartialEq`, `PartialOrd`, `Hash`, `Eq`, `Ord`: Support comparison and hashing, which is
///   necessary for certain data structures like sets or maps.
///
/// # Methods
///
/// The `State` trait defines several methods that must be implemented:
///
/// - `state_name`: Returns the name of the state as a string representation. Useful for identifying the current state.
/// - `input_data`: Returns a reference to the input data associated with the state. This data is used for processing within the state.
/// - `compute_output_data`: Performs computations to generate the output data from the input data. This method modifies the state to store the output data.
/// - `output_data`: Returns an optional reference to the output data. If the output data has been computed, it will return `Some(&OutputData)`, otherwise `None`.
/// - `has_output_data_been_computed`: Returns a boolean indicating whether the output data has been computed. The default implementation checks if `output_data` returns `Some`.
/// - `context_data`: Returns a reference to the context data associated with the state. This data provides additional information or settings relevant to the state.
pub trait State:
    Debug + Send + Sync + Unpin + Clone + PartialEq + PartialOrd + Hash + Eq + Ord
{
    type InputData: StateData;
    type OutputData: StateData;
    type Context: Context;

    /// Returns the name of the state.
    ///
    /// This method provides a way to identify the current state by name, which can be useful for debugging
    /// or logging purposes.
    ///
    /// # Returns
    ///
    /// A type that can be converted into a string, representing the name of the state.
    fn state_name(&self) -> impl ToString;

    /// Returns a reference to the input data associated with the state.
    ///
    /// Input data is used by the state to perform its operations and produce output data. This method
    /// provides access to the current input data.
    ///
    /// # Returns
    ///
    /// A reference to the input data of type `InputData`.
    fn input_data(&self) -> &Self::InputData;

    /// Computes the output data from the input data.
    ///
    /// This method is responsible for processing the input data and generating the corresponding output data.
    /// It modifies the state to store the computed output data.
    fn compute_output_data(&mut self);

    /// Returns an optional reference to the output data.
    ///
    /// This method provides access to the output data if it has been computed. If the output data has not
    /// been computed, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the output data of type `OutputData` if available, otherwise `None`.
    fn output_data(&self) -> Option<&Self::OutputData>;

    /// Checks if the output data has been computed.
    ///
    /// By default, this method checks if `output_data` returns `Some`. It can be overridden for more complex checks.
    ///
    /// # Returns
    ///
    /// `true` if the output data has been computed, otherwise `false`.
    fn has_output_data_been_computed(&self) -> bool {
        self.output_data().is_some()
    }

    /// Returns a reference to the context data associated with the state.
    ///
    /// Context data provides additional information or configuration that is relevant to the state. This method
    /// provides access to the current context data.
    ///
    /// # Returns
    ///
    /// A reference to the context data of type `Context`.
    fn context_data(&self) -> &Self::Context;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::{SampleState, SampleStateContext, SampleStateData};
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
