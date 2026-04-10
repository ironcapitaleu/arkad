//! # Phase Stream
//!
//! Defines the [`PhaseStream`] type alias for the boxed async stream returned by
//! [`ExtractSuperState::into_stream()`](super::ExtractSuperState).

use std::pin::Pin;

use futures_core::Stream;

/// A boxed, `Send`-able stream of phase completion results.
///
/// Each item is `Ok(String)` containing the `Display` snapshot of the state after
/// successful computation, or an error if the phase failed.
pub type PhaseStream =
    Pin<Box<dyn Stream<Item = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>>;
