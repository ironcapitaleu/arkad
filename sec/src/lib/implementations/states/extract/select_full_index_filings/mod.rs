//! # Select Full Index Filings State
//!
//! Provides the [`SelectFullIndexFilings`] state, which turns a stored full index into the list of
//! filings to fetch: one [`FilingRef`](crate::shared::filing_ref::FilingRef) per accession number, in index order, with a date filed in
//! the range of its context.
//!
//! A full index holds every row of its quarter, so the state drops the rows outside the range
//! first. It keeps the rows that [`select_filings_filed_in`] accepts: the forms in
//! [`SELECTED_FORMS`](crate::shared::financial::form::SELECTED_FORMS) and their amendments.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`SelectFullIndexFilingsContext`] carried alongside the state.
//! - [`data`]: The [`SelectFullIndexFilingsInput`] data type.

use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::{InvalidFiledDateRange, InvalidFullIndex};
use crate::implementations::states::extract::select_filings::SelectFilingsOutput;
use crate::shared::filing_ref::select_filings_filed_in;
use crate::shared::form_index_row::parse_form_index;
use crate::shared::quarter::Quarter;
use crate::traits::error::FromDomainError;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::SelectFullIndexFilingsContext;
pub use data::SelectFullIndexFilingsInput;

/// Selects the filings to fetch from a stored full index, for the dates filed of its context.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFullIndexFilings {
    input: SelectFullIndexFilingsInput,
    context: SelectFullIndexFilingsContext,
    output: Option<SelectFilingsOutput>,
}

impl SelectFullIndexFilings {
    /// Creates a new [`SelectFullIndexFilings`] state with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    /// use chrono::NaiveDate;
    /// use sec::implementations::states::extract::select_full_index_filings::*;
    /// use sec::shared::quarter::Quarter;
    /// use sec::shared::stored_form_index::StoredFormIndex;
    /// use state_maschine::prelude::State as SMState;
    /// use storage::Sha256Digest;
    /// use storage::implementations::repository::filesystem::DocumentKey;
    /// use uuid::Uuid;
    ///
    /// let index = StoredFormIndex {
    ///     key: DocumentKey::new("sec/full-index/2026/QTR4/form.idx")
    ///         .expect("Given a hardcoded valid relative path, the key should always build"),
    ///     sha256: Sha256Digest::of(b"---"),
    ///     text: Bytes::from_static(b"---"),
    /// };
    /// let day = NaiveDate::from_ymd_opt(2026, 10, 1)
    ///     .expect("Given a hardcoded calendar date, the date should always build");
    /// let state = SelectFullIndexFilings::new(
    ///     SelectFullIndexFilingsInput::new(index),
    ///     SelectFullIndexFilingsContext::new(Uuid::nil(), Quarter::of(day), day, day),
    /// );
    ///
    /// let expected_result = false;
    ///
    /// let result = state.has_output_data_been_computed();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(
        input: SelectFullIndexFilingsInput,
        context: SelectFullIndexFilingsContext,
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
        SelectFullIndexFilingsInput,
        Option<SelectFilingsOutput>,
        SelectFullIndexFilingsContext,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for SelectFullIndexFilings {
    /// Selects the filings of the stored index with a date filed in the range of the context, and
    /// stores them as output.
    ///
    /// Reads the index text with [`String::from_utf8_lossy`]. The parser takes the form from the
    /// first 17 bytes of a row and the other fields from its end, so a non-UTF-8 byte in a company
    /// name cannot change them.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::InvalidFiledDateRange`] if the first date filed of the context comes
    /// after its last date filed, or if a date is outside the quarter of the context. One full
    /// index lists one quarter, so such a range selects no filing or misses filings.
    ///
    /// Returns [`StateError::InvalidFullIndex`] if [`parse_form_index`] rejects the index text.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let (quarter, first, last) = (
            self.context.quarter,
            self.context.first_filed_date,
            self.context.last_filed_date,
        );
        if first > last || Quarter::of(first) != quarter || Quarter::of(last) != quarter {
            return Err(InvalidFiledDateRange::new(STATE_NAME, quarter, first, last).into());
        }
        let rows = parse_form_index(&String::from_utf8_lossy(&self.input.index.text))
            .map_err(|e| InvalidFullIndex::from_domain_error(STATE_NAME, e))?;
        let dates = first..=last;
        self.output = Some(SelectFilingsOutput::new(select_filings_filed_in(
            &rows, &dates,
        )));
        Ok(())
    }
}

impl SMState for SelectFullIndexFilings {
    type InputData = SelectFullIndexFilingsInput;
    type OutputData = SelectFilingsOutput;
    type Context = SelectFullIndexFilingsContext;

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

impl fmt::Display for SelectFullIndexFilings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            STATE_NAME,
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

    use bytes::Bytes;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use storage::Sha256Digest;
    use storage::implementations::repository::filesystem::DocumentKey;
    use uuid::Uuid;

    use super::*;
    use crate::shared::form_index_row::FormIndexError;
    use crate::shared::stored_form_index::StoredFormIndex;
    use crate::tests::fixtures::data::full_index::SAMPLE_FULL_INDEX_2026_Q3;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    /// Returns the quarter 2026 Q3, the quarter of the fixture.
    fn q3() -> Quarter {
        Quarter::of(date(2026, 7, 1))
    }

    fn test_state(text: &'static str, first: NaiveDate, last: NaiveDate) -> SelectFullIndexFilings {
        let index = StoredFormIndex {
            key: DocumentKey::new("sec/full-index/2026/QTR3/form.idx")
                .expect("Given a hardcoded valid relative path, the key should always build"),
            sha256: Sha256Digest::of(text.as_bytes()),
            text: Bytes::from_static(text.as_bytes()),
        };
        SelectFullIndexFilings::new(
            SelectFullIndexFilingsInput::new(index),
            SelectFullIndexFilingsContext::new(Uuid::nil(), q3(), first, last),
        )
    }

    async fn selected_accession_numbers(state: &mut SelectFullIndexFilings) -> Vec<String> {
        state.compute_output_data_async().await.expect(
            "Given the stored fixture of a real full index, the selection should always succeed",
        );
        state
            .output_data()
            .expect("Given a successful computation, the output should always be present")
            .filings
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn should_return_state_name_for_select_full_index_filings() {
        let state = test_state(
            SAMPLE_FULL_INDEX_2026_Q3,
            date(2026, 9, 30),
            date(2026, 9, 30),
        );

        let expected_result = "Select Full Index Filings";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_only_the_selected_filings_of_the_range_for_sample_full_index() {
        let mut state = test_state(
            SAMPLE_FULL_INDEX_2026_Q3,
            date(2026, 9, 30),
            date(2026, 9, 30),
        );

        let expected_result = vec![
            "0001493152-26-044953".to_string(),
            "0001193125-26-409138".to_string(),
            "0001170010-26-000104".to_string(),
            "0001104659-26-112415".to_string(),
        ];

        let result = selected_accession_numbers(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_index_order_when_the_range_spans_two_days() {
        let mut state = test_state(
            SAMPLE_FULL_INDEX_2026_Q3,
            date(2026, 9, 29),
            date(2026, 9, 30),
        );

        let expected_result = vec![
            "0001493152-26-044953".to_string(),
            "0001193125-26-409138".to_string(),
            "0001104659-26-111786".to_string(),
            "0001170010-26-000104".to_string(),
            "0001104659-26-112415".to_string(),
        ];

        let result = selected_accession_numbers(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_filed_date_range_when_the_first_filed_date_is_after_the_last() {
        let mut state = test_state(
            SAMPLE_FULL_INDEX_2026_Q3,
            date(2026, 9, 30),
            date(2026, 9, 29),
        );

        let expected_result = Err(StateError::InvalidFiledDateRange(
            InvalidFiledDateRange::new(STATE_NAME, q3(), date(2026, 9, 30), date(2026, 9, 29)),
        ));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_filed_date_range_when_the_dates_are_in_two_quarters() {
        let mut state = test_state(
            SAMPLE_FULL_INDEX_2026_Q3,
            date(2026, 9, 30),
            date(2026, 10, 1),
        );

        let expected_result = Err(StateError::InvalidFiledDateRange(
            InvalidFiledDateRange::new(STATE_NAME, q3(), date(2026, 9, 30), date(2026, 10, 1)),
        ));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_filed_date_range_when_the_dates_are_outside_the_quarter_of_the_index()
     {
        let mut state = test_state(
            SAMPLE_FULL_INDEX_2026_Q3,
            date(2026, 10, 1),
            date(2026, 10, 2),
        );

        let expected_result = Err(StateError::InvalidFiledDateRange(
            InvalidFiledDateRange::new(STATE_NAME, q3(), date(2026, 10, 1), date(2026, 10, 2)),
        ));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_only_the_row_of_the_day_when_the_range_is_one_day() {
        let mut state = test_state(
            SAMPLE_FULL_INDEX_2026_Q3,
            date(2026, 9, 29),
            date(2026, 9, 29),
        );

        let expected_result = vec!["0001104659-26-111786".to_string()];

        let result = selected_accession_numbers(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_full_index_when_the_index_has_no_header() {
        let mut state = test_state("no header line", date(2026, 9, 1), date(2026, 9, 30));

        let expected_result = Err(StateError::InvalidFullIndex(InvalidFullIndex::new(
            STATE_NAME,
            FormIndexError::MissingFormIndexHeader,
        )));

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = test_state(
            SAMPLE_FULL_INDEX_2026_Q3,
            date(2026, 9, 30),
            date(2026, 9, 30),
        );

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
    const fn should_implement_value_traits_for_select_full_index_filings() {
        implements_value_traits::<SelectFullIndexFilings>();
    }
}
