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
    async fn should_yield_three_items_when_streaming_three_state_pipeline() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();
        let mut stream = std::pin::pin!(sm.into_stream());

        let expected_result = 3;

        let mut count = 0;
        while let Some(result) = stream.next().await {
            result.expect("Each state in the streaming state machine fixture should succeed");
            count += 1;
        }
        let result = count;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_yield_items_in_state_order_when_streaming() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();
        let mut stream = std::pin::pin!(sm.into_stream());

        let expected_result = vec!["SampleStateA", "SampleStateB", "SampleStateC"];

        let mut result = Vec::new();
        while let Some(item) = stream.next().await {
            result.push(
                item.expect("Each state in the streaming state machine fixture should succeed"),
            );
        }

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_complete_stream_when_all_phases_succeed() {
        let sm = SampleStreamingSuperState::<SampleStateA>::new();
        let mut stream = std::pin::pin!(sm.into_stream());

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
