//! # State Machine Stream
//!
//! This module defines the [`IntoStateMachineStream`] trait, the [`NonTerminal`] marker trait,
//! and the [`StateMachineStream`] type alias for converting state machines into async streams
//! that drive their own state machine execution to completion.
//!
//! ## Design
//!
//! The [`NonTerminal`] trait marks a state machine at a specific state as non-terminal, declaring
//! which state it is currently at and which state it transitions to. The blanket implementation
//! of [`IntoStateMachineStream`] covers any state machine that implements [`NonTerminal`] and has
//! a [`Transition`] to its declared successor, where the state machine at that successor state is
//! also streamable. Only terminal states (no [`NonTerminal`] impl) need a manual implementation.

use std::pin::Pin;

use futures_core::Stream;

use super::state::State;
use super::transition::Transition;

/// A boxed, `Send`-able stream of state machine phase results.
///
/// Each item is `Ok(String)` containing the `Display` snapshot of the state after
/// successful computation, or an error if the phase failed.
pub type StateMachineStream =
    Pin<Box<dyn Stream<Item = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>>;

/// Marker trait declaring that a state machine is at a non-terminal state.
///
/// Each impl represents one edge in the state machine's transition graph,
/// pinning down the current state and its successor for streaming purposes.
///
/// A state machine can still have multiple [`Transition`] impls (branching, circular graphs).
/// [`NonTerminal`] declares which transition the streaming path follows.
pub trait NonTerminal {
    /// The state this state machine is currently at.
    type Current: State;
    /// The successor state that the stream will transition to.
    type Next: State;
}

/// Trait for converting a state machine into an async [`StateMachineStream`].
///
/// Implementors produce a stream that drives the state machine through its phases,
/// yielding a result after each phase completes.
pub trait IntoStateMachineStream {
    /// Consumes the state machine and returns a stream of phase results.
    fn into_stream(self) -> StateMachineStream;
}

/// Blanket implementation for any state machine at a non-terminal state. Computes the current
/// phase via the inner state, yields the result, transitions to the successor declared by
/// [`NonTerminal`], and chains the next state machine's stream.
impl<SM> IntoStateMachineStream for SM
where
    SM: NonTerminal + Transition<SM::Current, SM::Next> + Send + 'static,
    SM::NewStateMachine: IntoStateMachineStream + Send + 'static,
{
    fn into_stream(self) -> StateMachineStream {
        Box::pin(async_stream::stream! {
            let mut sm = self;
            #[allow(clippy::useless_conversion)]
            sm.current_state_mut().compute_output_data_async().await.map_err(|e| {
                let err: crate::error::State = e.into();
                Box::new(err) as Box<dyn std::error::Error + Send + Sync>
            })?;
            yield Ok(format!("{}", sm.current_state()));


            let next = sm.transition_to_next_state_sec()
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;


            let mut rest = std::pin::pin!(next.into_stream());
            while let Some(item) = futures_util::StreamExt::next(&mut rest).await {
                yield item;
            }
        })
    }
}
