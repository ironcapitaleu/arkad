//! # SEC State Machine Traits
//!
//! This module defines the core traits for building SEC-specific state machines, states, super states, and transitions.
//! It extends the generic [`state_maschine`] framework with domain-specific abstractions for robust, type-safe, and testable workflows
//! in SEC data processing pipelines.
//!
//! ## Modules
//! - [`state`]: Traits for defining states, context data, and state data used within SEC state machines.
//! - [`super_state`]: Traits for hierarchical (composite) states, supporting advanced state machine patterns.
//! - [`transition`]: Traits for modeling transitions between states, including error handling for transition failures.
//!
//! ## Usage
//! Implement the [`StateMachine`] trait for your SEC-specific state machine types to leverage the extensible framework
//! and integrate with concrete state, context, and data implementations found in [`crate::implementations`].
//!
//! See the documentation for each submodule for details on trait requirements and usage patterns.

use state_maschine::prelude::StateMachine as SMStateMachine;

pub mod state;
pub mod super_state;
pub mod transition;

use state::State;

/// The `StateMachine` trait is a marker trait for SEC-specific state machines,
/// extending the generic [`StateMachine`] trait with additional constraints for SEC domain states.
///
/// # Type Parameters
/// - `S`: The state type, which must implement the SEC [`State`] trait.
pub trait StateMachine<S>: SMStateMachine<S>
where
    S: State,
{
}
