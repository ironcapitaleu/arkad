use crate::state_machine::{StateMachine, state::State, transition::Transition};

use super::{FirstInnerState, SampleState, SampleSuperState, SecondInnerState};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct HierarchicalStateMachine<S: State> {
    current_state: S,
}

impl HierarchicalStateMachine<SampleSuperState<FirstInnerState>> {
    pub fn new() -> Self {
        Self {
            current_state: SampleSuperState::new(),
        }
    }
}

// impl StateMachine<SampleSuperState<FirstInnerState>> for HierarchicalStateMachine<SampleSuperState<FirstInnerState>> {
//     fn get_current_state(&self) -> &SampleSuperState<FirstInnerState> {
//         &self.current_state
//     }

//     fn get_current_state_mut(&mut self) -> &mut SampleSuperState<FirstInnerState> {
//         &mut self.current_state
//     }

//     fn run(&mut self) {
//         println!(
//             "Running state: {}",
//             self.get_current_state().get_state_name().to_string()
//         );
//     }

//     fn advance_state(&mut self) {
//         println!("Advancing state");
//         self.get_current_state_mut().compute_output_data();
//     }
// }

// impl StateMachine<SampleSuperState<SecondInnerState>> for HierarchicalStateMachine<SampleSuperState<SecondInnerState>> {
//     fn get_current_state(&self) -> &SampleSuperState<SecondInnerState> {
//         &self.current_state
//     }

//     fn get_current_state_mut(&mut self) -> &mut SampleSuperState<SecondInnerState> {
//         &mut self.current_state
//     }

//     fn run(&mut self) {
//         println!(
//             "Running state: {}",
//             self.get_current_state().get_state_name().to_string()
//         );
//     }

//     fn advance_state(&mut self) {
//         println!("Advancing state");
//         self.get_current_state_mut().compute_output_data();
//     }
// }

impl<S: State> StateMachine<S> for HierarchicalStateMachine<S> {
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

impl Transition<SampleSuperState<FirstInnerState>, SampleSuperState<SecondInnerState>>
    for HierarchicalStateMachine<SampleSuperState<FirstInnerState>>
{
    type NewStateMachine = HierarchicalStateMachine<SampleSuperState<SecondInnerState>>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        let inner_transition_result_state =
            Transition::<FirstInnerState, SecondInnerState>::transition_to_next_state(
                self.current_state,
            )
            .expect("Should not fail the transitions to 'SecondInnerState'.");

        Ok(HierarchicalStateMachine {
            current_state: inner_transition_result_state,
        })
    }
}

impl Transition<SampleSuperState<SecondInnerState>, SampleState>
    for HierarchicalStateMachine<SampleSuperState<SecondInnerState>>
{
    type NewStateMachine = HierarchicalStateMachine<SampleState>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Ok(HierarchicalStateMachine {
            current_state: SampleState::default(),
        })
    }
}
