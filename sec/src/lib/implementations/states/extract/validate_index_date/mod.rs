//! # Validate Index Date State
//!
//! Provides the [`ValidateIndexDate`] state, which turns a raw date string such as `2026-10-27`
//! into a validated [`IndexDate`].
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`ValidateIndexDateContext`] carried alongside the state.
//! - [`data`]: The [`ValidateIndexDateInput`] and [`ValidateIndexDateOutput`] data types.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::validate_index_date::*;
//! use sec::prelude::*;
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let input = ValidateIndexDateInput::new("2026-10-27");
//! let context = ValidateIndexDateContext::new(Uuid::new_v4());
//! let mut state = ValidateIndexDate::new(input, context);
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("Given a hardcoded past date, validation should always succeed");
//!
//! let expected_result = "QTR4";
//!
//! let result = state
//!     .output_data()
//!     .expect("Given a successful computation, the output should always be present")
//!     .index_date
//!     .quarter();
//!
//! assert_eq!(result, expected_result);
//! # }
//! ```

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::InvalidIndexDate;
use crate::shared::index_date::IndexDate;
use crate::traits::error::FromDomainError;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::ValidateIndexDateContext;
pub use data::ValidateIndexDateInput;
pub use data::ValidateIndexDateOutput;

/// Validates a raw date string into an [`IndexDate`].
///
/// The validation accepts only a date that [`IndexDate::new`] accepts.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct ValidateIndexDate {
    input: ValidateIndexDateInput,
    context: ValidateIndexDateContext,
    output: Option<ValidateIndexDateOutput>,
}

impl ValidateIndexDate {
    /// Creates a new [`ValidateIndexDate`] state with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::validate_index_date::*;
    /// use state_maschine::prelude::State as SMState;
    /// use uuid::Uuid;
    ///
    /// let input = ValidateIndexDateInput::new("2026-10-27");
    /// let context = ValidateIndexDateContext::new(Uuid::nil());
    /// let state = ValidateIndexDate::new(input, context);
    ///
    /// let expected_result = false;
    ///
    /// let result = state.has_output_data_been_computed();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(input: ValidateIndexDateInput, context: ValidateIndexDateContext) -> Self {
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
        ValidateIndexDateInput,
        Option<ValidateIndexDateOutput>,
        ValidateIndexDateContext,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for ValidateIndexDate {
    /// Validates the raw date and stores the [`IndexDate`] as output.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::InvalidIndexDate`] if [`IndexDate::new`] rejects the raw date.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let index_date = IndexDate::new(&self.input.raw_date)
            .map_err(|e| InvalidIndexDate::from_domain_error(STATE_NAME, e))?;
        self.output = Some(ValidateIndexDateOutput::new(index_date));
        Ok(())
    }
}

impl SMState for ValidateIndexDate {
    type InputData = ValidateIndexDateInput;
    type OutputData = ValidateIndexDateOutput;
    type Context = ValidateIndexDateContext;

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

impl fmt::Display for ValidateIndexDate {
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
    use crate::shared::index_date::{IndexDateError, InvalidIndexDateReason};

    fn test_state(raw_date: &str) -> ValidateIndexDate {
        ValidateIndexDate::new(
            ValidateIndexDateInput::new(raw_date),
            ValidateIndexDateContext::new(Uuid::nil()),
        )
    }

    #[test]
    fn should_return_state_name_for_validate_index_date() {
        let state = test_state("2026-10-27");

        let expected_result = "Validate Index Date";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = test_state("2026-10-27");

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_index_date_when_raw_date_is_valid() {
        let mut state = test_state("2026-10-27");

        let expected_result = Some(&ValidateIndexDateOutput::new(
            IndexDate::new("2026-10-27")
                .expect("Given a hardcoded past date, parsing should always succeed"),
        ));

        state
            .compute_output_data_async()
            .await
            .expect("Given a past date, validation should always succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_the_index_date_when_raw_date_has_surrounding_whitespace() {
        let mut state = test_state(" 2026-10-27 ");

        let expected_result = Some(&ValidateIndexDateOutput::new(
            IndexDate::new("2026-10-27")
                .expect("Given a hardcoded past date, parsing should always succeed"),
        ));

        state
            .compute_output_data_async()
            .await
            .expect("Given a past date with whitespace, validation should always succeed");
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_index_date_when_raw_date_is_malformed() {
        let mut state = test_state("2026-1-2");

        let expected_result = StateError::InvalidIndexDate(InvalidIndexDate::new(
            STATE_NAME,
            IndexDateError::new(InvalidIndexDateReason::InvalidIndexDateFormat, "2026-1-2"),
        ));

        let result = state
            .compute_output_data_async()
            .await
            .expect_err("Given a date without zero padding, validation should always fail");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_future_index_date_when_raw_date_is_after_today() {
        let mut state = test_state("9999-12-31");

        let expected_result = true;

        let error = state
            .compute_output_data_async()
            .await
            .expect_err("Given a date in year 9999, validation should always fail");
        let result = matches!(
            error,
            StateError::InvalidIndexDate(InvalidIndexDate {
                index_date_error: IndexDateError {
                    reason: InvalidIndexDateReason::FutureIndexDate { .. },
                    ..
                },
                ..
            })
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_output_empty_when_raw_date_is_malformed() {
        let mut state = test_state("2026-1-2");

        let expected_result = None;

        let _ = state.compute_output_data_async().await;
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = test_state("2026-10-27");

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_output_when_calling_sync_compute_outside_tokio_runtime() {
        let mut state = test_state("2026-10-27");

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "compute_output_data failed")]
    fn should_panic_when_calling_sync_compute_with_malformed_raw_date() {
        let mut state = test_state("2026-1-2");

        state.compute_output_data();
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_validate_index_date() {
        implements_value_traits::<ValidateIndexDate>();
    }
}
