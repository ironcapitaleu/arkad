use crate::state_machine::state::State;
use state_maschine::prelude::Transition as SMTransition;

pub trait Transition<T, U>: SMTransition<T, U>
where
    T: State,
    U: State,
{
}
