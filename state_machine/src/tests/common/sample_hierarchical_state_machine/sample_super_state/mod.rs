use crate::state_machine::{
    StateMachine, state::State, super_state::SuperState, transition::Transition,
};
pub mod first_inner_state;
pub mod sample_super_state_context;
pub mod sample_super_state_data;
pub mod second_inner_state;

pub use first_inner_state::FirstInnerState;
pub use sample_super_state_context::SampleSuperStateContext;
pub use sample_super_state_data::SampleSuperStateData;
pub use second_inner_state::SecondInnerState;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SampleSuperState<S: State> {
    current_state: S,
    input: SampleSuperStateData,
    output: Option<SampleSuperStateData>,
    context_data: SampleSuperStateContext,
}

impl SampleSuperState<FirstInnerState> {
    pub fn new() -> Self {
        Self {
            current_state: FirstInnerState::default(),
            input: SampleSuperStateData::default(),
            output: None,
            context_data: SampleSuperStateContext::default(),
        }
    }
}

impl<S: State> SuperState<S> for SampleSuperState<S> {}

impl<S: State> State for SampleSuperState<S> {
    type InputData = SampleSuperStateData;
    type OutputData = SampleSuperStateData;
    type Context = SampleSuperStateContext;

    fn get_state_name(&self) -> impl ToString {
        "Super State"
    }

    fn get_input_data(&self) -> &SampleSuperStateData {
        &self.input
    }

    fn compute_output_data(&mut self) {
        self.output = Some(SampleSuperStateData::default());
    }

    fn get_output_data(&self) -> Option<&SampleSuperStateData> {
        self.output.as_ref()
    }

    fn get_context_data(&self) -> &SampleSuperStateContext {
        &self.context_data
    }
}

impl<S: State> StateMachine<S> for SampleSuperState<S> {
    fn get_current_state(&self) -> &S {
        &self.current_state
    }

    fn get_current_state_mut(&mut self) -> &mut S {
        &mut self.current_state
    }

    fn run(&mut self) {
        println!(
            "Running state: {}",
            self.get_current_state().get_state_name().to_string()
        );
    }

    fn advance_state(&mut self) {
        println!("Advancing state");
        self.get_current_state_mut().compute_output_data();
    }
}

impl Transition<FirstInnerState, SecondInnerState> for SampleSuperState<FirstInnerState> {
    type NewStateMachine = SampleSuperState<SecondInnerState>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Ok(SampleSuperState {
            input: SampleSuperStateData::default(),
            output: None,
            context_data: SampleSuperStateContext::default(),
            current_state: SecondInnerState::default(),
        })
    }
}
