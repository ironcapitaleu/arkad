use std::fmt::Debug;

use crate::state_machine::{StateMachine, state::State};
use crate::tests::common::SampleState;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct SimpleStateMachine {
    current_state: SampleState,
}

impl StateMachine<SampleState> for SimpleStateMachine {
    fn current_state(&self) -> &SampleState {
        &self.current_state
    }

    fn current_state_mut(&mut self) -> &mut SampleState {
        &mut self.current_state
    }

    fn run(&mut self) {
        // Example implementation of run method
        // In a real scenario, this would contain logic to perform operations based on the current state
        println!(
            "Running state: {}",
            self.current_state().state_name().to_string()
        );
    }

    fn advance_state(&mut self) {
        // Example implementation to change state
        // Here we could implement logic to transition to another state
        println!("Advancing state");
        // Example: Simulate state change by computing output data
        self.current_state_mut().compute_output_data();
    }
}
