//! # State Machine Traits
//!
//! Provides the SEC-specific state machine traits, layered on the generic [`state_maschine`]
//! framework.
//!
//! This module's own [`StateMachine`] trait ties the pieces together; its submodules define the
//! states, transitions, and hierarchy that a machine is built from.
//!
//! ## Modules
//!
//! - [`state`]: The [`State`] trait plus its context and state-data traits.
//! - [`super_state`]: The [`SuperState`](super_state::SuperState) trait for hierarchical states.
//! - [`transition`]: The [`Transition`](transition::Transition) trait for moving between states.
//! - [`stream`]: Converting a state machine into an async event stream.

use std::fmt::Display;

use state_maschine::prelude::StateMachine as SMStateMachine;

pub mod state;
pub mod stream;
pub mod super_state;
pub mod transition;

use state::State;

/// Marker trait for SEC state machines, refining the generic [`SMStateMachine`] to SEC states.
///
/// Adds no methods; it bundles the bounds an SEC machine must satisfy ([`Display`] plus a state
/// type implementing the SEC [`State`] trait) into one nameable contract.
///
/// # Type Parameters
///
/// - `S`: The active state type. Must implement [`State`].
pub trait StateMachine<S>: SMStateMachine<S> + Display
where
    S: State,
{
}
