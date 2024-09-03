use crate::state_machine::state::State;

pub mod second_inner_state_context;
pub mod second_inner_state_data;

pub use second_inner_state_context::SecondInnerStateContext;
pub use second_inner_state_data::SecondInnerStateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SecondInnerState {
    input: SecondInnerStateData,
    output: Option<SecondInnerStateData>,
    context_data: SecondInnerStateContext,
}

impl State for SecondInnerState {
    type InputData = SecondInnerStateData;
    type OutputData = SecondInnerStateData;
    type Context = SecondInnerStateContext;

    fn get_state_name(&self) -> impl ToString {
        "Second Inner State"
    }

    fn get_input_data(&self) -> &SecondInnerStateData {
        &self.input
    }

    fn compute_output_data(&mut self) {
        self.output = Some(SecondInnerStateData::default());
    }

    fn get_output_data(&self) -> Option<&SecondInnerStateData> {
        self.output.as_ref()
    }

    fn get_context_data(&self) -> &SecondInnerStateContext {
        &self.context_data
    }
}
