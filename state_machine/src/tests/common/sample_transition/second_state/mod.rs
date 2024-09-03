use crate::state_machine::state::State;

pub mod second_state_context;
pub mod second_state_data;

pub use second_state_context::SecondStateContext;
pub use second_state_data::SecondStateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SecondState {
    input: SecondStateData,
    output: Option<SecondStateData>,
    context_data: SecondStateContext,
}

impl State for SecondState {
    type InputData = SecondStateData;
    type OutputData = SecondStateData;
    type Context = SecondStateContext;

    fn get_state_name(&self) -> impl ToString {
        "Second State"
    }

    fn get_input_data(&self) -> &SecondStateData {
        &self.input
    }

    fn compute_output_data(&mut self) {
        self.output = Some(SecondStateData::default());
    }

    fn get_output_data(&self) -> Option<&SecondStateData> {
        self.output.as_ref()
    }

    fn get_context_data(&self) -> &SecondStateContext {
        &self.context_data
    }
}
