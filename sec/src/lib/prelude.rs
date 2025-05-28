//! # SEC Prelude Module
//!
//! This module provides convenient re-exports of the core SEC state machine traits for downstream crates.
//! By importing `sec::prelude::*`, users gain easy access to the essential building blocks for defining,
//! composing, and interacting with SEC state machines, states, context, and transitions.
//!
//! ## Re-exports
//! - [`SecStateMachine`](crate::traits::state_machine::SecStateMachine): Trait for SEC-specific state machines.
//! - [`State`](crate::traits::state_machine::state::State): Trait for defining individual states.
//! - [`StateData`](crate::traits::state_machine::state::StateData): Trait for state data management.
//! - [`ContextData`](crate::traits::state_machine::state::ContextData): Trait for context data management.
//! - [`SuperState`](crate::traits::state_machine::super_state::SuperState): Trait for hierarchical state machines.
//! - [`Transition`](crate::traits::state_machine::transition::Transition): Trait for defining state transitions.
//!
//! ## Usage
//!
//! ```rust
//! use sec::prelude::*;
//! // Now you can use State, StateData, ContextData, etc. directly.
//! ```

pub use crate::traits::state_machine::SecStateMachine;
pub use crate::traits::state_machine::state::ContextData;
pub use crate::traits::state_machine::state::State;
pub use crate::traits::state_machine::state::StateData;
pub use crate::traits::state_machine::super_state::SuperState;
pub use crate::traits::state_machine::transition::Transition;
