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

use std::fmt;
use std::pin::Pin;

use futures_core::Stream;

use state_maschine::prelude::{State as SMState, StateMachine as SMStateMachine};

use super::state::State;
use super::transition::Transition;

/// Events that occur during state machine stream execution.
///
/// Each variant represents a specific moment in the state machine lifecycle.
/// Displays as `snake_case` for structured logging compatibility.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamEvent {
    /// A state is about to begin computation.
    StateStarted,
    /// A state has successfully completed computation.
    StateCompleted,
    /// A state's computation has failed.
    StateFailed,
    /// A transition to the next state has completed successfully.
    TransitionCompleted,
    /// A transition to the next state has failed.
    TransitionFailed,
}

impl fmt::Display for StreamEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StateStarted => write!(f, "state_started"),
            Self::StateCompleted => write!(f, "state_completed"),
            Self::StateFailed => write!(f, "state_failed"),
            Self::TransitionCompleted => write!(f, "state_transition_completed"),
            Self::TransitionFailed => write!(f, "state_transition_failed"),
        }
    }
}

/// A structured item yielded by the state machine stream on success.
///
/// Carries the event type, state name, and serialized state data for structured logging.
#[derive(Debug, Clone)]
pub struct StreamItem {
    /// What happened in the state machine lifecycle.
    pub event: StreamEvent,
    /// The name of the state this event relates to.
    pub state_name: String,
    /// Serialized state data at the time of the event.
    pub data: serde_json::Value,
}

/// An error yielded by the state machine stream on failure.
///
/// Wraps the existing [`StateMachine`](crate::error::StateMachine) error with stream context
/// (execution ID, state name, event type, serialized state data).
#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub struct StreamError {
    /// What was happening when the error occurred.
    pub event: StreamEvent,
    /// The execution ID for this pipeline run.
    pub execution_id: uuid::Uuid,
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// Serialized state data at the time of the error.
    pub data: serde_json::Value,
    /// The underlying state machine error.
    #[source]
    pub source: crate::error::StateMachine,
}

/// A boxed, `Send`-able stream of state machine execution results.
///
/// Each item is `Ok(StreamItem)` for successful events or `Err(StreamError)` for failures.
pub type StateMachineStream = Pin<Box<dyn Stream<Item = Result<StreamItem, StreamError>> + Send>>;

/// Marker trait declaring that a state machine is at a non-terminal state.
///
/// Each impl represents one edge in the state machine's transition graph,
/// pinning down the current state and its successor for streaming purposes.
///
/// A state machine can still have multiple [`Transition`] impls (branching, circular graphs).
/// [`NonTerminal`] declares which transition the streaming path follows.
pub trait NonTerminal {
    /// The state this state machine is currently at.
    type Current: State + serde::Serialize;
    /// The successor state that the stream will transition to.
    type Next: State;
}

/// Trait for converting a state machine into an async [`StateMachineStream`].
///
/// Implementors produce a stream that drives the state machine through its phases,
/// yielding structured [`StreamItem`]s on success and [`StreamError`]s on failure.
pub trait IntoStateMachineStream {
    /// Consumes the state machine and returns a stream of execution results.
    ///
    /// The `execution_id` identifies this pipeline run and is forwarded through
    /// the entire chain of states, including across nested super states.
    fn into_stream(self, execution_id: uuid::Uuid) -> StateMachineStream;
}

/// Blanket implementation for any state machine at a non-terminal state. Yields
/// [`StreamItem`]s for `StateStarted`, `StateCompleted`, and `TransitionCompleted`,
/// then chains the next state machine's stream with the same `execution_id`.
impl<SM> IntoStateMachineStream for SM
where
    SM: NonTerminal + Transition<SM::Current, SM::Next> + Send + 'static,
    SM::NewStateMachine: IntoStateMachineStream + Send + 'static,
{
    fn into_stream(self, execution_id: uuid::Uuid) -> StateMachineStream {
        Box::pin(async_stream::stream! {
            let mut sm = self;
            let state_name = sm.current_state().state_name().to_string();

            // StateStarted
            yield Ok(StreamItem {
                event: StreamEvent::StateStarted,
                state_name: state_name.clone(),
                data: serde_json::to_value(sm.current_state()).unwrap_or_default(),
            });

            // Compute — convert error immediately to release the mutable borrow on sm
            let compute_err: Option<crate::error::StateMachine> = {
                let result = sm.current_state_mut().compute_output_data_async().await;
                match result {
                    Ok(()) => None,
                    Err(e) => {
                        let state_err: crate::error::State = e.into();
                        Some(state_err.into())
                    }
                }
            };

            if let Some(sm_error) = compute_err {
                yield Err(StreamError {
                    event: StreamEvent::StateFailed,
                    execution_id,
                    state_name,
                    data: serde_json::to_value(sm.current_state()).unwrap_or_default(),
                    source: sm_error,
                });
                return;
            }

            yield Ok(StreamItem {
                event: StreamEvent::StateCompleted,
                state_name: state_name.clone(),
                data: serde_json::to_value(sm.current_state()).unwrap_or_default(),
            });

            // Transition
            let from_name = state_name;
            match sm.transition_to_next_state_sec() {
                Ok(next) => {
                    let to_name = next.current_state().state_name().to_string();
                    yield Ok(StreamItem {
                        event: StreamEvent::TransitionCompleted,
                        state_name: from_name,
                        data: serde_json::json!({ "to": to_name }),
                    });

                    // Chain — forward same execution_id
                    let mut rest = std::pin::pin!(next.into_stream(execution_id));
                    while let Some(item) = futures_util::StreamExt::next(&mut rest).await {
                        yield item;
                    }
                }
                Err(e) => {
                    yield Err(StreamError {
                        event: StreamEvent::TransitionFailed,
                        execution_id,
                        state_name: from_name,
                        data: serde_json::Value::Null,
                        source: e.into(),
                    });
                    return;
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use futures_util::StreamExt;
    use pretty_assertions::assert_eq;

    use crate::prelude::*;
    use crate::tests::fixtures::sample_streaming_super_state::{
        SampleStateA, SampleStateC, SampleStreamingContext, SampleStreamingData,
        SampleStreamingSuperState,
    };

    use super::IntoStateMachineStream;

    // --- Functional tests ---

    #[test]
    fn should_return_super_state_name_when_in_initial_streaming_state() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();

        let expected_result = "Sample Streaming SuperState (Current: Sample State A)";

        let result = sm.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_inner_state_name_when_accessing_current_state() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();

        let expected_result = "Sample State A";

        let result = sm.current_state().state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_have_no_output_for_inner_state_before_compute_on_super_state() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();

        let expected_result = true;

        let result = sm.current_state().output_data().is_none();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_have_output_for_inner_state_after_compute_on_super_state() {
        let mut sm = SampleStreamingSuperState::<SampleStateA>::new();
        sm.compute_output_data_async()
            .await
            .expect("Hardcoded fixture state should always compute output successfully");

        let expected_result = true;

        let result = sm.current_state().output_data().is_some();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_context_data_when_in_initial_streaming_state() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();

        let expected_result = &SampleStreamingContext;

        let result = sm.context_data();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_default_input_data_when_in_initial_streaming_state() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();

        let expected_result = &SampleStreamingData;

        let result = sm.input_data();

        assert_eq!(result, expected_result);
    }

    // --- Streaming tests ---

    #[tokio::test]
    async fn should_yield_eight_items_when_streaming_three_state_pipeline() {
        // 3 states: (Started + Completed + Transition) x 2 non-terminal + (Started + Completed) x 1 terminal = 8
        let sm = SampleStreamingSuperState::<SampleStateA>::new();
        let execution_id = uuid::Uuid::new_v4();
        let mut stream = std::pin::pin!(sm.into_stream(execution_id));

        let expected_result = 8;

        let mut count = 0;
        while let Some(result) = stream.next().await {
            result.expect("Each event in the streaming state machine fixture should succeed");
            count += 1;
        }
        let result = count;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_events_in_correct_order_when_streaming() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();
        let execution_id = uuid::Uuid::new_v4();
        let mut stream = std::pin::pin!(sm.into_stream(execution_id));

        let expected_result = vec![
            super::StreamEvent::StateStarted,
            super::StreamEvent::StateCompleted,
            super::StreamEvent::TransitionCompleted,
            super::StreamEvent::StateStarted,
            super::StreamEvent::StateCompleted,
            super::StreamEvent::TransitionCompleted,
            super::StreamEvent::StateStarted,
            super::StreamEvent::StateCompleted,
        ];

        let mut result = Vec::new();
        while let Some(item) = stream.next().await {
            let item =
                item.expect("Each event in the streaming state machine fixture should succeed");
            result.push(item.event);
        }

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_complete_stream_when_all_phases_succeed() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();
        let execution_id = uuid::Uuid::new_v4();
        let mut stream = std::pin::pin!(sm.into_stream(execution_id));

        let expected_result = true;

        let mut all_ok = true;
        while let Some(result) = stream.next().await {
            if result.is_err() {
                all_ok = false;
            }
        }
        let result = all_ok;

        assert_eq!(result, expected_result);
    }

    // --- Trait compliance: StateMachineStream ---
    // Note: StateMachineStream is Send but NOT Sync — async streams hold mutable
    // state across await points. You move a stream to a consumer, not share it.

    const fn assert_send<T: Send>() {}

    #[test]
    const fn should_produce_send_stream_when_into_stream_is_called() {
        assert_send::<super::StateMachineStream>();
    }

    // --- Trait compliance: non-terminal SampleStreamingSuperState<SampleStateA> ---

    const fn assert_sync<T: Sync>() {}
    const fn assert_unpin<T: Unpin>() {}
    const fn assert_sized<T: Sized>() {}
    const fn assert_debug<T: Debug>() {}
    const fn assert_clone<T: Clone>() {}
    const fn assert_hash<T: Hash>() {}
    const fn assert_partial_eq<T: PartialEq>() {}
    const fn assert_eq<T: Eq>() {}
    const fn assert_partial_ord<T: PartialOrd>() {}
    const fn assert_ord<T: Ord>() {}

    #[test]
    const fn should_implement_send_for_non_terminal_streaming_super_state() {
        assert_send::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_sync_for_non_terminal_streaming_super_state() {
        assert_sync::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_unpin_for_non_terminal_streaming_super_state() {
        assert_unpin::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_sized_for_non_terminal_streaming_super_state() {
        assert_sized::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_debug_for_non_terminal_streaming_super_state() {
        assert_debug::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_clone_for_non_terminal_streaming_super_state() {
        assert_clone::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_hash_for_non_terminal_streaming_super_state() {
        assert_hash::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_partial_eq_for_non_terminal_streaming_super_state() {
        assert_partial_eq::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_eq_for_non_terminal_streaming_super_state() {
        assert_eq::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_partial_ord_for_non_terminal_streaming_super_state() {
        assert_partial_ord::<SampleStreamingSuperState<SampleStateA>>();
    }

    #[test]
    const fn should_implement_ord_for_non_terminal_streaming_super_state() {
        assert_ord::<SampleStreamingSuperState<SampleStateA>>();
    }

    // --- Trait compliance: terminal SampleStreamingSuperState<SampleStateC> ---

    #[test]
    const fn should_implement_send_for_terminal_streaming_super_state() {
        assert_send::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_sync_for_terminal_streaming_super_state() {
        assert_sync::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_unpin_for_terminal_streaming_super_state() {
        assert_unpin::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_sized_for_terminal_streaming_super_state() {
        assert_sized::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_debug_for_terminal_streaming_super_state() {
        assert_debug::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_clone_for_terminal_streaming_super_state() {
        assert_clone::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_hash_for_terminal_streaming_super_state() {
        assert_hash::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_partial_eq_for_terminal_streaming_super_state() {
        assert_partial_eq::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_eq_for_terminal_streaming_super_state() {
        assert_eq::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_partial_ord_for_terminal_streaming_super_state() {
        assert_partial_ord::<SampleStreamingSuperState<SampleStateC>>();
    }

    #[test]
    const fn should_implement_ord_for_terminal_streaming_super_state() {
        assert_ord::<SampleStreamingSuperState<SampleStateC>>();
    }
}
