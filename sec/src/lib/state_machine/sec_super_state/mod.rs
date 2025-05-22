use crate::state_machine::state::State;
use state_maschine::prelude::*;

pub trait SecSuperState<S>: StateMachine<S>
where
    S: State,
{
}
