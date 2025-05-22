use state_maschine::prelude::Transition as SMTransition;

use crate::state_machine::state::State;

pub trait Transition<T, U>: SMTransition<T, U>
where
    T: State,
    U: State,
{
}
