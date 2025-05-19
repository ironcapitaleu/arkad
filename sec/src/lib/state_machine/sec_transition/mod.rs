use crate::state_machine::sec_state::SecState;
use state_maschine::prelude::*;

pub trait SecTransition<T, U>: Transition<T, U>
where
    T: SecState,
    U: SecState,
{
}
