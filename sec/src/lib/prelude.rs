//! The `sec` imports various traits defined in this crate.
//!
//! The intention is that one can include `use sec::prelude::*` and
//! have easy access to the various traits and methods you will need.

pub use crate::traits::state_machine::SecStateMachine;
pub use crate::traits::state_machine::state::ContextData;
pub use crate::traits::state_machine::state::State;
pub use crate::traits::state_machine::state::StateData;
pub use crate::traits::state_machine::super_state::SuperState;
pub use crate::traits::state_machine::transition::Transition;
