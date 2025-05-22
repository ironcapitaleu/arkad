use state_maschine::prelude::*;

pub mod extract;
pub mod sec_error;
pub mod state;
pub mod sec_super_state;
pub mod sec_transition;

use state::SecState;

pub trait SecStateMachine<S>: StateMachine<S>
where
    S: SecState,
{
}
