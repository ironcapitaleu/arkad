use state_maschine::prelude::*;

pub mod extract;
pub mod sec_error;
pub mod sec_super_state;
pub mod sec_transition;
pub mod state;

use state::State;

pub trait SecStateMachine<S>: StateMachine<S>
where
    S: State,
{
}
