//! # Validate Notes Period State
//!
//! Provides the [`ValidateNotesPeriod`] state, which turns a raw period string such as `2024q1`
//! into a validated [`NotesPeriod`].
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`ValidateNotesPeriodContext`] carried alongside the state.
//! - [`data`]: The [`ValidateNotesPeriodInput`] and [`ValidateNotesPeriodOutput`] data types.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::validate_notes_period::*;
//! use sec::prelude::*;
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! # use sec::shared::raw_document::RawDocument;
//! # use sec::shared::raw_source::RawNotesArchive;
//! # use storage::implementations::repository::filesystem::FilesystemRepository;
//! # let repository = FilesystemRepository::builder()
//! #     .writer()
//! #     .document_type::<RawDocument<RawNotesArchive>>()
//! #     .root("data")
//! #     .build();
//! let input = ValidateNotesPeriodInput::new("2024q1");
//! let context = ValidateNotesPeriodContext::new(Uuid::new_v4(), repository);
//! let mut state = ValidateNotesPeriod::new(input, context);
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("Given a hardcoded published quarter, validation should always succeed");
//!
//! let expected_result = "2024q1";
//!
//! let result = state
//!     .output_data()
//!     .expect("Given a successful computation, the output should always be present")
//!     .notes_period
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
use crate::error::state_machine::state::InvalidNotesPeriod;
use crate::shared::notes_period::NotesPeriod;
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::RawNotesArchive;
use crate::traits::error::FromDomainError;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::ValidateNotesPeriodContext;
pub use data::ValidateNotesPeriodInput;
pub use data::ValidateNotesPeriodOutput;

/// Validates a raw period string into a [`NotesPeriod`].
///
/// The validation accepts only a period with a published Notes file, as [`NotesPeriod::new`]
/// defines it.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ValidateNotesPeriod<R> {
    input: ValidateNotesPeriodInput,
    context: ValidateNotesPeriodContext<R>,
    output: Option<ValidateNotesPeriodOutput>,
}

impl<R> ValidateNotesPeriod<R> {
    /// Creates a new [`ValidateNotesPeriod`] state with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::validate_notes_period::*;
    /// use state_maschine::prelude::State as SMState;
    /// use uuid::Uuid;
    ///
    /// let input = ValidateNotesPeriodInput::new("2024q1");
    /// # use sec::shared::raw_document::RawDocument;
    /// # use sec::shared::raw_source::RawNotesArchive;
    /// # use storage::implementations::repository::filesystem::FilesystemRepository;
    /// # let repository = FilesystemRepository::builder()
    /// #     .writer()
    /// #     .document_type::<RawDocument<RawNotesArchive>>()
    /// #     .root("data")
    /// #     .build();
    /// let context = ValidateNotesPeriodContext::new(Uuid::nil(), repository);
    /// let state = ValidateNotesPeriod::new(input, context);
    ///
    /// let expected_result = false;
    ///
    /// let result = state.has_output_data_been_computed();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        input: ValidateNotesPeriodInput,
        context: ValidateNotesPeriodContext<R>,
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
        ValidateNotesPeriodInput,
        Option<ValidateNotesPeriodOutput>,
        ValidateNotesPeriodContext<R>,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl<R: RawRepository<RawNotesArchive>> State for ValidateNotesPeriod<R> {
    /// Validates the raw period and stores the [`NotesPeriod`] as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::InvalidNotesPeriod`] if [`NotesPeriod::new`] rejects the raw period.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let notes_period = NotesPeriod::new(&self.input.raw_period)
            .map_err(|e| InvalidNotesPeriod::from_domain_error(STATE_NAME, e))?;
        self.output = Some(ValidateNotesPeriodOutput::new(notes_period));
        Ok(())
    }
}

impl<R: RawRepository<RawNotesArchive>> SMState for ValidateNotesPeriod<R> {
    type InputData = ValidateNotesPeriodInput;
    type OutputData = ValidateNotesPeriodOutput;
    type Context = ValidateNotesPeriodContext<R>;

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

impl<R: RawRepository<RawNotesArchive>> fmt::Display for ValidateNotesPeriod<R> {
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
    use crate::shared::notes_period::{InvalidNotesPeriodReason, NotesPeriodError};
    use crate::tests::fixtures::sample_write_repository::FakeNotesRepository;

    fn test_state(raw_period: &str) -> ValidateNotesPeriod<FakeNotesRepository> {
        ValidateNotesPeriod::new(
            ValidateNotesPeriodInput::new(raw_period),
            ValidateNotesPeriodContext::new(Uuid::nil(), FakeNotesRepository::new()),
        )
    }

    #[test]
    fn should_return_state_name_for_validate_notes_period() {
        let state = test_state("2024q1");

        let expected_result = "Validate Notes Period";

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
    async fn should_output_the_notes_period_when_raw_period_is_published() {
        let mut state = test_state("2024q1");

        let expected_result = Some(&ValidateNotesPeriodOutput::new(
            NotesPeriod::new("2024q1")
                .expect("Given a hardcoded published quarter, parsing should always succeed"),
        ));

        state
            .compute_output_data_async()
            .await
            .expect("Given a published quarter, validation should always succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_notes_period_when_raw_period_has_surrounding_whitespace() {
        let mut state = test_state(" 2024q1 ");

        let expected_result = Some(&ValidateNotesPeriodOutput::new(
            NotesPeriod::new("2024q1")
                .expect("Given a hardcoded published quarter, parsing should always succeed"),
        ));

        state
            .compute_output_data_async()
            .await
            .expect("Given a published quarter with whitespace, validation should always succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_notes_period_when_raw_period_has_no_valid_form() {
        let mut state = test_state("2024");

        let expected_result = StateError::InvalidNotesPeriod(InvalidNotesPeriod::new(
            STATE_NAME,
            NotesPeriodError::new(InvalidNotesPeriodReason::InvalidNotesPeriodFormat, "2024"),
        ));

        let result = state
            .compute_output_data_async()
            .await
            .expect_err("Given a period without quarter or month, validation should always fail");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_output_empty_when_raw_period_has_no_valid_form() {
        let mut state = test_state("2024");

        let expected_result = None;

        let _ = state.compute_output_data_async().await;
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

    #[test]
    #[should_panic(expected = "compute_output_data failed")]
    fn should_panic_when_calling_sync_compute_with_invalid_raw_period() {
        let mut state = test_state("2024");

        state.compute_output_data();
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_validate_notes_period() {
        implements_value_traits::<ValidateNotesPeriod<FakeNotesRepository>>();
    }
}
