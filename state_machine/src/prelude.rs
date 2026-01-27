//! The `state_maschine` imports the various traits defined in this crate.
//!
//! The intention is that one can include `use state_maschine::prelude::*` and
//! have easy access to the various traits and methods you will need.

pub use crate::state_machine::StateMachine;
pub use crate::state_machine::state::Context;
pub use crate::state_machine::state::State;
pub use crate::state_machine::state::StateData;
pub use crate::state_machine::super_state::SuperState;
pub use crate::state_machine::transition::Transition;
