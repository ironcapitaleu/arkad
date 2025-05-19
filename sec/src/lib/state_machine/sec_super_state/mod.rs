use crate::state_machine::sec_state::SecState;
use state_maschine::prelude::*;

pub trait SecSuperState<S>: StateMachine<S>
where
    S: SecState,
{
}
