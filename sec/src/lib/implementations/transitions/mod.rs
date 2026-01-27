//! # Transitions Module
//!
//! This module contains implementations of state transitions for the SEC state machine.
//! Transitions are organized by source state, with each source state having a dedicated submodule
//! containing transitions to various destination states.
//!
//! ## Module Structure
//! - [`from`]: Contains transition implementations organized by source state

pub mod from;
