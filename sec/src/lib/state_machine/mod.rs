use state_maschine::prelude::*;

pub mod extract;
pub mod sec_error;
pub mod sec_state;
pub mod sec_super_state;
pub mod sec_transition;

use sec_state::SecState;

pub trait SecStateMachine<S>: StateMachine<S>
where
    S: SecState,
{
}
