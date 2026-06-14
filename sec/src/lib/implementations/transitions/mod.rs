//! # Transitions
//!
//! Provides the state-to-state transitions that advance the SEC pipeline.
//!
//! A transition converts a completed state into the next one, carrying its output forward as the
//! successor's input and context. They are grouped by *source* state so that the conversions
//! leaving any given state are all found in one place.
//!
//! ## Modules
//!
//! - [`from`]: Transitions grouped by the state they originate from.

pub mod from;
