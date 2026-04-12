//! # Sample Streaming Super State Fixture
//!
//! A minimal 3-state linear pipeline (`StateA → StateB → StateC`) for testing the
//! [`IntoStateMachineStream`] blanket impl and [`NonTerminal`] trait.

use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::{
    State as SMState, StateMachine as SMStateMachine, SuperState as SMSuperState,
    Transition as SMTransition,
};

use crate::error::State as StateError;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::prelude::*;
use crate::traits::state_machine::state::StateData;

pub mod state_a;
pub mod state_b;
pub mod state_c;

pub use state_a::SampleStateA;
pub use state_b::SampleStateB;
pub use state_c::SampleStateC;

// --- Shared data/context unit types ---

/// Unit struct for super state data — no actual data needed for streaming tests.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord, serde::Serialize)]
pub struct SampleStreamingData;

impl StateData for SampleStreamingData {
    fn update_state(&mut self, _updates: Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

impl SMStateData for SampleStreamingData {
    type UpdateType = ();

    fn state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, _updates: Self::UpdateType) {}
}

/// Unit struct for super state context — no actual context needed for streaming tests.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord, serde::Serialize)]
pub struct SampleStreamingContext;

impl Context for SampleStreamingContext {
    fn max_retries(&self) -> u32 {
        0
    }
}

impl SMContext for SampleStreamingContext {
    type UpdateType = ();

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, _updates: Self::UpdateType) {}
}

// --- Super state ---

/// A minimal super state for testing streaming. Generic over the current inner state.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord, serde::Serialize)]
pub struct SampleStreamingSuperState<S: State> {
    current_state: S,
    input: SampleStreamingData,
    output: Option<SampleStreamingData>,
    context: SampleStreamingContext,
}

impl SampleStreamingSuperState<SampleStateA> {
    /// Creates a new streaming super state starting at `SampleStateA`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_state: SampleStateA::new(),
            input: SampleStreamingData,
            output: None,
            context: SampleStreamingContext,
        }
    }
}

impl<S: State> fmt::Display for SampleStreamingSuperState<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SampleStreamingSuperState({})",
            self.current_state.state_name().to_string()
        )
    }
}

#[async_trait]
impl<S: State> State for SampleStreamingSuperState<S> {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.current_state
            .compute_output_data_async()
            .await
            .map_err(Into::into)
    }
}

impl<S: State> SMState for SampleStreamingSuperState<S> {
    type InputData = SampleStreamingData;
    type OutputData = SampleStreamingData;
    type Context = SampleStreamingContext;

    fn state_name(&self) -> impl ToString {
        format!(
            "Sample Streaming SuperState (Current: {})",
            self.current_state.state_name().to_string()
        )
    }

    fn compute_output_data(&mut self) {}

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl<S: State> SMStateMachine<S> for SampleStreamingSuperState<S> {
    fn current_state(&self) -> &S {
        &self.current_state
    }

    fn current_state_mut(&mut self) -> &mut S {
        &mut self.current_state
    }

    fn run(&mut self) {}

    fn advance_state(&mut self) {}
}

impl<S: State> StateMachine<S> for SampleStreamingSuperState<S> {}

impl<S: State> SMSuperState<S> for SampleStreamingSuperState<S> {}

impl<S: State> SuperState<S> for SampleStreamingSuperState<S> {}

// --- Transitions ---

impl SMTransition<SampleStateA, SampleStateB> for SampleStreamingSuperState<SampleStateA> {
    type NewStateMachine = SampleStreamingSuperState<SampleStateB>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err("Use transition_to_next_state_sec() for rich error handling")
    }
}

impl Transition<SampleStateA, SampleStateB> for SampleStreamingSuperState<SampleStateA> {
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(SampleStreamingSuperState {
            current_state: SampleStateB::new(),
            input: SampleStreamingData,
            output: None,
            context: SampleStreamingContext,
        })
    }
}

impl SMTransition<SampleStateB, SampleStateC> for SampleStreamingSuperState<SampleStateB> {
    type NewStateMachine = SampleStreamingSuperState<SampleStateC>;

    fn transition_to_next_state(self) -> Result<Self::NewStateMachine, &'static str> {
        Err("Use transition_to_next_state_sec() for rich error handling")
    }
}

impl Transition<SampleStateB, SampleStateC> for SampleStreamingSuperState<SampleStateB> {
    fn transition_to_next_state_sec(self) -> Result<Self::NewStateMachine, TransitionError> {
        Ok(SampleStreamingSuperState {
            current_state: SampleStateC::new(),
            input: SampleStreamingData,
            output: None,
            context: SampleStreamingContext,
        })
    }
}

// --- Streaming ---

impl NonTerminal for SampleStreamingSuperState<SampleStateA> {
    type Current = SampleStateA;
    type Next = SampleStateB;
}

impl NonTerminal for SampleStreamingSuperState<SampleStateB> {
    type Current = SampleStateB;
    type Next = SampleStateC;
}

/// Terminal state — manual [`IntoStateMachineStream`] impl.
impl IntoStateMachineStream for SampleStreamingSuperState<SampleStateC> {
    fn into_stream(self, execution_id: uuid::Uuid) -> StateMachineStream {
        Box::pin(async_stream::stream! {
            use crate::traits::state_machine::stream::{StreamEvent, StreamItem, StreamError};

            let mut sm = self;
            let state_name = sm.current_state().state_name().to_string();

            let state_start = std::time::Instant::now();

            yield Ok(StreamItem {
                event: StreamEvent::StateStarted,
                state_name: state_name.clone(),
                data: serde_json::to_value(sm.current_state()).unwrap_or_default(),
                event_duration: std::time::Duration::ZERO,
            });

            match sm.current_state_mut().compute_output_data_async().await {
                Ok(()) => {
                    yield Ok(StreamItem {
                        event: StreamEvent::StateCompleted,
                        state_name,
                        data: serde_json::to_value(sm.current_state()).unwrap_or_default(),
                        event_duration: state_start.elapsed(),
                    });
                }
                Err(e) => {
                    #[allow(clippy::useless_conversion)]
                    let state_err: crate::error::State = e.into();
                    let sm_error: crate::error::StateMachine = state_err.into();
                    yield Err(StreamError {
                        event: StreamEvent::StateFailed,
                        execution_id,
                        state_name,
                        data: serde_json::to_value(sm.current_state()).unwrap_or_default(),
                        source: sm_error,
                    });
                }
            }
        })
    }
}
