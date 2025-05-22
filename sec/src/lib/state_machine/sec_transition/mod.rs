use crate::state_machine::state::State;
use state_maschine::prelude::*;

pub trait SecTransition<T, U>: Transition<T, U>
where
    T: State,
    U: State,
{
}
