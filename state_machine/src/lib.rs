//! # State Machine
//!
//! A generic, trait-based framework for building state machines in Rust.
//!
//! The crate defines the abstract pieces of a state machine, states, their context and data,
//! transitions between them, and hierarchical super-states, as a set of traits. Downstream crates
//! (such as `sec`) implement these traits to model their own domain workflows.
//!
//! ## Modules
//!
//! - [`state_machine`]: The core traits: [`StateMachine`](state_machine::StateMachine),
//!   [`State`](state_machine::state::State), [`SuperState`](state_machine::super_state::SuperState),
//!   and [`Transition`](state_machine::transition::Transition).
//! - [`prelude`]: Re-exports of those traits for convenient glob import.

pub mod prelude;

pub mod state_machine;

#[cfg(test)]
mod tests;
