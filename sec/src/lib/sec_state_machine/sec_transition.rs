use state_machine::prelude::*;

pub trait SecTransition<S>: Transition<S>
where
    S: SecState,
{

}