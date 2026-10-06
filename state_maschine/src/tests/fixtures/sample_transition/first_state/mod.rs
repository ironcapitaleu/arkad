use crate::state_machine::state::State;

pub mod first_state_context;
pub mod first_state_data;

pub use first_state_context::FirstStateContext;
pub use first_state_data::FirstStateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct FirstState {
    input: FirstStateData,
    output: Option<FirstStateData>,
    context_data: FirstStateContext,
}

impl State for FirstState {
    type InputData = FirstStateData;
    type OutputData = FirstStateData;
    type Context = FirstStateContext;

    fn state_name(&self) -> impl ToString {
        "First State"
    }

    fn input_data(&self) -> &FirstStateData {
        &self.input
    }

    fn compute_output_data(&mut self) {
        self.output = Some(FirstStateData::default());
    }

    fn output_data(&self) -> Option<&FirstStateData> {
        self.output.as_ref()
    }

    fn context_data(&self) -> &FirstStateContext {
        &self.context_data
    }
}
