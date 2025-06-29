//! # SEC Prelude Module
//!
//! This module provides convenient re-exports of the core SEC state machine traits for downstream crates.
//! By importing `sec::prelude::*`, users gain easy access to the essential building blocks for defining,
//! composing, and interacting with SEC state machines, states, context, and transitions.
//!
//! ## Re-exports
//! - [`StateMachine`]: Trait for SEC-specific state machines.
//! - [`State`]: Trait for defining individual states.
//! - [`StateData`]: Trait for state data management.
//! - [`ContextData`]: Trait for context data management.
//! - [`SuperState`]: Trait for hierarchical state machines.
//! - [`Transition`]: Trait for defining state transitions.
//!
//! ## Usage
//!
//! ```rust
//! use sec::prelude::*;
//! // Now you can use State, StateData, ContextData, etc. directly.
//! ```

pub use crate::traits::state_machine::StateMachine;
pub use crate::traits::state_machine::state::ContextData;
pub use crate::traits::state_machine::state::State;
pub use crate::traits::state_machine::state::StateData;
pub use crate::traits::state_machine::super_state::SuperState;
pub use crate::traits::state_machine::transition::Transition;

pub use state_maschine::prelude::ContextData as SMContextData;
pub use state_maschine::prelude::State as SMState;
pub use state_maschine::prelude::StateData as SMStateData;
pub use state_maschine::prelude::StateMachine as SMStateMachine;
pub use state_maschine::prelude::SuperState as SMSuperState;
pub use state_maschine::prelude::Transition as SMTransition;
