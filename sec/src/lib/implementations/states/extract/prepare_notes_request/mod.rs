//! # Prepare Notes Request State
//!
//! Provides the [`PrepareNotesRequest`] state, which builds the [`SecRequest`] for the Notes file
//! of one [`NotesPeriod`](crate::shared::notes_period::NotesPeriod).
//!
//! The state makes no network call.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`PrepareNotesRequestContext`] carried alongside the state.
//! - [`data`]: The [`PrepareNotesRequestInput`] and [`PrepareNotesRequestOutput`] data types.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::prepare_notes_request::*;
//! use sec::prelude::*;
//! use sec::shared::notes_period::NotesPeriod;
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! # let repository = ();
//! let period = NotesPeriod::new("2024q1")
//!     .expect("Given a hardcoded published quarter, parsing should always succeed");
//! let input = PrepareNotesRequestInput::new(period);
//! let context = PrepareNotesRequestContext::new(Uuid::new_v4(), repository);
//! let mut state = PrepareNotesRequest::new(input, context);
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("Given a validated period, the preparation should always succeed");
//!
//! let expected_result = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip";
//!
//! let result = state
//!     .output_data()
//!     .expect("Given a successful computation, the output should always be present")
//!     .request
//!     .inner
//!     .url()
//!     .to_string();
//!
//! assert_eq!(result, expected_result);
//! # }
//! ```

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::implementations::states::extract::notes_repository::NotesRepository;
use crate::shared::request::implementations::sec_request::SecRequest;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::PrepareNotesRequestContext;
pub use data::PrepareNotesRequestInput;
pub use data::PrepareNotesRequestOutput;

/// Builds the [`SecRequest`] for the Notes file of one period.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct PrepareNotesRequest<R> {
    input: PrepareNotesRequestInput,
    context: PrepareNotesRequestContext<R>,
    output: Option<PrepareNotesRequestOutput>,
}

impl<R> PrepareNotesRequest<R> {
    /// Creates a new [`PrepareNotesRequest`] state with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::prepare_notes_request::*;
    /// use sec::shared::notes_period::NotesPeriod;
    /// use state_maschine::prelude::State as SMState;
    /// use uuid::Uuid;
    ///
    /// let period = NotesPeriod::new("2024q1")
    ///     .expect("Given a hardcoded published quarter, parsing should always succeed");
    /// let input = PrepareNotesRequestInput::new(period);
    /// let context = PrepareNotesRequestContext::new(Uuid::nil(), ());
    /// let state = PrepareNotesRequest::new(input, context);
    ///
    /// let expected_result = false;
    ///
    /// let result = state.has_output_data_been_computed();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        input: PrepareNotesRequestInput,
        context: PrepareNotesRequestContext<R>,
    ) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        PrepareNotesRequestInput,
        Option<PrepareNotesRequestOutput>,
        PrepareNotesRequestContext<R>,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl<R: NotesRepository> State for PrepareNotesRequest<R> {
    /// Builds the Notes file request for the input period and stores it as output.
    ///
    /// # Errors
    ///
    /// Never returns an error. The [`Result`] is part of the [`State`] contract.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let request = SecRequest::builder()
            .notes_file()
            .period(self.input.notes_period)
            .build();
        self.output = Some(PrepareNotesRequestOutput::new(request));
        Ok(())
    }
}

impl<R: NotesRepository> SMState for PrepareNotesRequest<R> {
    type InputData = PrepareNotesRequestInput;
    type OutputData = PrepareNotesRequestOutput;
    type Context = PrepareNotesRequestContext<R>;

    fn state_name(&self) -> impl ToString {
        STATE_NAME
    }

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// # Panics
    ///
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime, because `block_in_place` requires a
    ///   multi-thread runtime.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            panic!("compute_output_data failed: {e}")
        }
    }

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

impl<R: NotesRepository> fmt::Display for PrepareNotesRequest<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::shared::notes_period::NotesPeriod;
    use crate::tests::fixtures::sample_write_repository::FakeWriteRepository;

    fn notes_period(period: &str) -> NotesPeriod {
        NotesPeriod::new(period)
            .expect("Given a hardcoded published period, parsing should always succeed")
    }

    fn test_state(period: &str) -> PrepareNotesRequest<FakeWriteRepository> {
        PrepareNotesRequest::new(
            PrepareNotesRequestInput::new(notes_period(period)),
            PrepareNotesRequestContext::new(Uuid::nil(), FakeWriteRepository::new()),
        )
    }

    #[test]
    fn should_return_state_name_for_prepare_notes_request() {
        let state = test_state("2024q1");

        let expected_result = "Prepare Notes Request";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = test_state("2024q1");

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_notes_file_request_of_the_input_period() {
        let mut state = test_state("2025_10");

        let expected_result = Some(&PrepareNotesRequestOutput::new(
            SecRequest::builder()
                .notes_file()
                .period(notes_period("2025_10"))
                .build(),
        ));

        state
            .compute_output_data_async()
            .await
            .expect("Given a validated period, the preparation should always succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = test_state("2024q1");

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_output_when_calling_sync_compute_outside_tokio_runtime() {
        let mut state = test_state("2024q1");

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_prepare_notes_request() {
        implements_value_traits::<PrepareNotesRequest<FakeWriteRepository>>();
    }
}
