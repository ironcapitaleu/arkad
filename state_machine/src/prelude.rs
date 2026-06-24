//! # Prelude
//!
//! Re-exports the crate's core traits for glob import.
//!
//! Bring them all into scope with `use state_maschine::prelude::*`.

pub use crate::state_machine::StateMachine;
pub use crate::state_machine::state::Context;
pub use crate::state_machine::state::State;
pub use crate::state_machine::state::StateData;
pub use crate::state_machine::super_state::SuperState;
pub use crate::state_machine::transition::Transition;
