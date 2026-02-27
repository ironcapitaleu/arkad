use crate::state_machine::{StateMachine, state::State, transition::Transition};

use super::{FirstState, SecondState};

// Define the StateMachine that uses FirstState and SecondState
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ComplexStateMachine<S: State> {
    current_state: S,
}

impl ComplexStateMachine<FirstState> {
    pub fn new() -> Self {
        Self {
            current_state: FirstState::default(),
        }
    }
}

impl StateMachine<FirstState> for ComplexStateMachine<FirstState> {
    fn current_state(&self) -> &FirstState {
        &self.current_state
    }

    fn current_state_mut(&mut self) -> &mut FirstState {
        &mut self.current_state
    }

    fn run(&mut self) {
        println!(
            "Running state: {}",
            self.current_state().state_name().to_string()
        );
    }

    fn advance_state(&mut self) {
        println!("Advancing state");
        self.current_state_mut().compute_output_data();
    }
}

impl StateMachine<SecondState> for ComplexStateMachine<SecondState> {
    fn current_state(&self) -> &SecondState {
        &self.current_state
    }

    fn current_state_mut(&mut self) -> &mut SecondState {
        &mut self.current_state
    }

    fn run(&mut self) {
        println!(
            "Running state: {}",
            self.current_state().state_name().to_string()
        );
    }

    fn advance_state(&mut self) {
        println!("Advancing state");
        self.current_state_mut().compute_output_data();
    }
}

impl Transition<FirstState, FirstState> for ComplexStateMachine<FirstState> {
    type NewStateMachine = Self;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Ok(Self {
            current_state: FirstState::default(),
        })
    }
}

impl Transition<FirstState, SecondState> for ComplexStateMachine<FirstState> {
    type NewStateMachine = ComplexStateMachine<SecondState>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Ok(ComplexStateMachine {
            current_state: SecondState::default(),
        })
    }
}
