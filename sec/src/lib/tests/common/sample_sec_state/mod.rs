use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::traits::state_machine::state::State;

pub mod sec_context;
pub mod sec_data;

pub use sec_context::SampleSecStateContext;
pub use sec_data::SampleSecStateInputData;
pub use sec_data::SampleSecStateOutputData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSecState {
    input: SampleSecStateInputData,
    context: SampleSecStateContext,
    output: Option<SampleSecStateOutputData>,
}

impl SampleSecState {
    #[must_use]
    pub const fn new(input: SampleSecStateInputData, context: SampleSecStateContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }
}

#[async_trait]
impl State for SampleSecState {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.output = Some(SampleSecStateOutputData {
            output_data: "Hello World!".to_string(),
        });
        Ok(())
    }
}

impl SMState for SampleSecState {
    type InputData = SampleSecStateInputData;
    type OutputData = SampleSecStateOutputData;
    type Context = SampleSecStateContext;

    fn get_state_name(&self) -> impl ToString {
        "Sample SEC State"
    }

    fn compute_output_data(&mut self) {}

    fn get_context_data(&self) -> &Self::Context {
        &self.context
    }

    fn get_input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn get_output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl fmt::Display for SampleSecState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.get_state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};
    use tokio;

    #[test]
    fn should_return_name_of_validation_state_when_in_validation_state() {
        let validation_state = SampleSecState::default();

        let expected_result = String::from("Sample SEC State");

        let result = validation_state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_validation_data_struct_as_input_data_when_in_initial_validation_state()
    {
        let validation_state = SampleSecState::default();

        let expected_result = &SampleSecStateInputData::default();

        let result = validation_state.get_input_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_state() {
        let validation_state = SampleSecState::default();

        let _result = validation_state
            .get_output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let validation_state = SampleSecState::default();

        let expected_result = false;

        let result = validation_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_state() {
        let validation_state = SampleSecState::default();

        let expected_result = &SampleSecStateContext::default();

        let result = validation_state.get_context_data();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_state_trait() {
        implements_auto_traits::<SampleSecState>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_state_trait() {
        implements_send::<SampleSecState>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_state_trait() {
        implements_sync::<SampleSecState>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_state_trait() {
        implements_send::<SampleSecState>();
        implements_sync::<SampleSecState>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_state_trait() {
        implements_sized::<SampleSecState>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_state_trait() {
        implements_hash::<SampleSecState>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_state_trait() {
        implements_partial_eq::<SampleSecState>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_state_trait() {
        implements_eq::<SampleSecState>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_state_trait() {
        implements_partial_ord::<SampleSecState>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_state_trait() {
        implements_ord::<SampleSecState>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_state_trait() {
        implements_default::<SampleSecState>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_state_trait() {
        implements_debug::<SampleSecState>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_state_trait() {
        implements_clone::<SampleSecState>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_state_trait() {
        implements_unpin::<SampleSecState>();
    }

    #[test]
    fn should_return_default_context_data_when_called_with_state_reference() {
        let validation_state = &SampleSecState::default();
        let ref_to_validation_state = &SampleSecState::default();

        let expected_result = validation_state.get_context_data();

        let result = ref_to_validation_state.get_context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_reference_state_has_not_computed_the_output() {
        let ref_to_validation_state = &mut SampleSecState::default();

        let expected_result = false;

        let result = ref_to_validation_state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "output should not be empty")]
    fn should_panic_when_trying_to_access_output_data_before_it_has_been_computed_in_reference_state()
     {
        let ref_to_validation_state = &SampleSecState::default();

        let _result = ref_to_validation_state
            .get_output_data()
            .expect("The output should not be empty.");
    }

    #[test]
    fn should_return_name_of_validation_state_when_calling_reference_to_validation_state() {
        let ref_to_validation_state = &SampleSecState::default();

        let expected_result = String::from("Sample SEC State");

        let result = ref_to_validation_state.get_state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_state_data_as_input_data_when_reference_validation_state_in_initial_state()
     {
        let ref_to_validation_state = &SampleSecState::default();

        let expected_result = &SampleSecStateInputData::default();

        let result = ref_to_validation_state.get_input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_not_change_input_data_when_computing_output_data() {
        let mut validation_state = SampleSecState::default();

        let expected_result = &validation_state.get_input_data().clone();

        validation_state
            .compute_output_data_async()
            .await
            .expect("Default state should always compute output data.");
        let result = validation_state.get_input_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_correct_output_data_when_computing_output_data() {
        let mut validation_state = SampleSecState::default();

        let expected_result = &SampleSecStateOutputData::default();

        validation_state
            .compute_output_data_async()
            .await
            .expect("Default state should always compute output data.");

        let result = validation_state.get_output_data().unwrap();

        assert_eq!(result, expected_result);
    }
}
