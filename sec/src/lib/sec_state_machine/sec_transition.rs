use state_maschine::prelude::*;
use crate::sec_state_machine::sec_state::SecState;

pub trait SecTransition<T, U>: Transition<T, U>
where
    T: SecState,
    U: SecState,
{

}