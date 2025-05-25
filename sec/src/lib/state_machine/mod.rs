use state_maschine::prelude::*;

pub mod extract;
pub mod state;
pub mod super_state;
pub mod transition;

use state::State;

pub trait SecStateMachine<S>: StateMachine<S>
where
    S: State,
{
}
