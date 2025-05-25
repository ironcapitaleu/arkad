use state_maschine::prelude::SuperState as SMSuperState;

use crate::traits::state_machine::state::State;

pub trait SuperState<S>: SMSuperState<S>
where
    S: State,
{
}
