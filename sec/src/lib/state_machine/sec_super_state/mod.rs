use crate::state_machine::state::SecState;
use state_maschine::prelude::*;

pub trait SecSuperState<S>: StateMachine<S>
where
    S: SecState,
{
}
