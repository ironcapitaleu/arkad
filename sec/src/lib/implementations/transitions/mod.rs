//! # Transitions
//!
//! Provides the state-to-state transitions that advance the SEC pipeline.
//!
//! A transition attempts to convert a source state into a successor state by assembling the
//! successor state's input and context from the source state's data. Transitions are grouped by
//! *source* state so that the conversions leaving any given state are all found in one place.
//!
//! ## Modules
//!
//! - [`from`]: Transitions grouped by the state they originate from.

pub mod from;
