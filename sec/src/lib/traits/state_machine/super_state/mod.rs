//! # `SuperState` Trait
//!
//! This module defines the [`SuperState`] trait for hierarchical (composite) states in the SEC state machine framework.
//! A `SuperState` is a state that itself acts as a state machine, enabling advanced patterns such as nested workflows,
//! composite states, and encapsulated sub-state machines within a parent state machine.
//!
//! ## Overview
//! The [`SuperState`] trait extends the generic [`state_maschine::prelude::SuperState`] trait, adding SEC-specific
//! constraints and integration with the [`State`] and [`super::StateMachine`] traits from this crate. This allows for robust, type-safe, and testable
//! hierarchical state machines in SEC data processing pipelines.
//!
//! ## Usage
//! Implement this trait for any state that should also be able to function as its own state machine, managing its own internal states
//! while participating as a single state in a parent state machine.
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::State`]: Trait for defining individual states.
//! - [`crate::traits::state_machine::StateMachine`]: Trait for SEC-specific state machines.
//! - [`state_maschine::state_machine::super_state::SuperState`]: Underlying framework trait for super states.
//!

use state_maschine::prelude::SuperState as SMSuperState;

use crate::traits::state_machine::state::State;

/// The `SuperState` trait is used for hierarchical (composite) states in the SEC state machine framework.
///
/// This trait extends the generic [`state_maschine::state_machine::super_state::SuperState`] trait, but restricts the state type `S`
/// to types that implement the SEC-specific [`State`] trait. This ensures that all sub-states within a `SuperState`
/// conform to the SEC state machine's requirements.
///
/// # Type Parameters
/// - `S`: The state type, which must implement [`State`].
pub trait SuperState<S>: SMSuperState<S>
where
    S: State,
{
}
