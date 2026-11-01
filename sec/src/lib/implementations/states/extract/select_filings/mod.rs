//! # Select Filings State
//!
//! Provides the [`SelectFilings`] state, which turns a stored daily index into the list of
//! filings to fetch: one [`FilingRef`] per accession number, in index order.
//!
//! The state keeps the rows that [`FilingRef::try_from`] accepts: the forms in
//! [`SELECTED_FORMS`](crate::shared::financial::form::SELECTED_FORMS) and their amendments. A
//! filing with several filers has one row per filer, and the state keeps the first one.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`SelectFilingsContext`] carried alongside the state.
//! - [`data`]: The [`SelectFilingsInput`] and [`SelectFilingsOutput`] data types.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::select_filings::*;
//! use sec::prelude::*;
//! use uuid::Uuid;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let input = SelectFilingsInput::new(None);
//! let context = SelectFilingsContext::new(Uuid::new_v4());
//! let mut state = SelectFilings::new(input, context);
//! state
//!     .compute_output_data_async()
//!     .await
//!     .expect("Given a day without filings, the selection should always succeed");
//!
//! let expected_result = 0;
//!
//! let result = state
//!     .output_data()
//!     .expect("Given a successful computation, the output should always be present")
//!     .filings
//!     .len();
//!
//! assert_eq!(result, expected_result);
//! # }
//! ```

use std::collections::HashSet;
use std::fmt;

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::error::state_machine::state::InvalidDailyIndex;
use crate::shared::daily_index_row::{DailyIndexError, parse_daily_index};
use crate::shared::filing_ref::FilingRef;
use crate::traits::error::FromDomainError;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::SelectFilingsContext;
pub use data::SelectFilingsInput;
pub use data::SelectFilingsOutput;

/// Selects the filings to fetch from a stored daily index.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SelectFilings {
    input: SelectFilingsInput,
    context: SelectFilingsContext,
    output: Option<SelectFilingsOutput>,
}

impl SelectFilings {
    /// Creates a new [`SelectFilings`] state with no output computed yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::implementations::states::extract::select_filings::*;
    /// use state_maschine::prelude::State as SMState;
    /// use uuid::Uuid;
    ///
    /// let input = SelectFilingsInput::new(None);
    /// let context = SelectFilingsContext::new(Uuid::nil());
    /// let state = SelectFilings::new(input, context);
    ///
    /// let expected_result = false;
    ///
    /// let result = state.has_output_data_been_computed();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn new(input: SelectFilingsInput, context: SelectFilingsContext) -> Self {
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
        SelectFilingsInput,
        Option<SelectFilingsOutput>,
        SelectFilingsContext,
    ) {
        (self.input, self.output, self.context)
    }
}

#[async_trait]
impl State for SelectFilings {
    /// Selects the filings of the stored index and stores them as output. A day without filings
    /// gives an empty list.
    ///
    /// Reads the index text with [`String::from_utf8_lossy`]. The parser takes the form from the
    /// first 17 bytes of a row and the other fields from its end, so a non-UTF-8 byte in a company
    /// name cannot change them.
    ///
    /// # Errors
    ///
    /// Returns [`StateError::InvalidDailyIndex`] if [`parse_daily_index`] rejects the index text.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let filings = match &self.input.index {
            Some(index) => selected_filings(&String::from_utf8_lossy(&index.text))
                .map_err(|e| InvalidDailyIndex::from_domain_error(STATE_NAME, e))?,
            None => Vec::new(),
        };
        self.output = Some(SelectFilingsOutput::new(filings));
        Ok(())
    }
}

impl SMState for SelectFilings {
    type InputData = SelectFilingsInput;
    type OutputData = SelectFilingsOutput;
    type Context = SelectFilingsContext;

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

impl fmt::Display for SelectFilings {
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

/// Parses the index text and keeps the first selected row of each accession number, in index
/// order.
fn selected_filings(text: &str) -> Result<Vec<FilingRef>, DailyIndexError> {
    let mut seen_accession_numbers = HashSet::new();
    Ok(parse_daily_index(text)?
        .iter()
        .filter_map(|row| FilingRef::try_from(row).ok())
        .filter(|filing| seen_accession_numbers.insert(filing.accession_number().clone()))
        .collect())
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use bytes::Bytes;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::DocumentKey;
    use uuid::Uuid;

    use super::*;
    use crate::implementations::states::extract::store_daily_index::StoredDailyIndex;
    use crate::shared::cik::Cik;
    use crate::shared::daily_index_row::InvalidDailyIndexLineReason;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::tests::fixtures::data::daily_index::SAMPLE_DAILY_INDEX;

    /// The header of a hand-written daily index. The last line starts with `---`.
    const INDEX_HEADER: &str = "Form Type   Company Name\n---\n";

    fn test_state(index_bytes: Option<Vec<u8>>) -> SelectFilings {
        let index = index_bytes.map(|text| StoredDailyIndex {
            key: DocumentKey::new("sec/daily-index/form.20261027.idx")
                .expect("Given a hardcoded valid relative path, the key should always build"),
            sha256: storage::Sha256Digest::of(&text),
            text: Bytes::from(text),
        });
        SelectFilings::new(
            SelectFilingsInput::new(index),
            SelectFilingsContext::new(Uuid::nil()),
        )
    }

    /// Returns one row line with the form in its 17-character column, as the SEC writes it.
    fn index_line(form_text: &str, cik: &str, accession_number: &str) -> String {
        format!(
            "{form_text:<17}EXAMPLE CORP{:<50}{cik:<12}20261030    edgar/data/{cik}/{accession_number}.txt\n",
            ""
        )
    }

    fn filing_ref(
        cik: &str,
        accession_number: &str,
        parsed_form: ParsedForm,
        filed_date: NaiveDate,
    ) -> FilingRef {
        FilingRef::new(
            Cik::new(cik).expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new(accession_number).expect("Given a hardcoded accession number of the right shape, creation should always succeed"),
            parsed_form,
            filed_date,
        )
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    async fn computed_filings(state: &mut SelectFilings) -> Vec<FilingRef> {
        state
            .compute_output_data_async()
            .await
            .expect("Given a well-formed index or no index, the selection should always succeed");
        state
            .output_data()
            .expect("Given a successful computation, the output should always be present")
            .filings
            .clone()
    }

    #[test]
    fn should_return_state_name_for_select_filings() {
        let state = test_state(None);

        let expected_result = "Select Filings";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = test_state(None);

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_one_filing_per_accession_number_in_index_order_for_sample_daily_index() {
        let index_text = SAMPLE_DAILY_INDEX.to_string();
        let mut state = test_state(Some(index_text.into_bytes()));

        let expected_result = vec![
            filing_ref(
                "1702744",
                "0001702744-26-000031",
                ParsedForm::new(Form::TenK, false),
                date(2026, 10, 27),
            ),
            filing_ref(
                "1053507",
                "0001053507-26-000157",
                ParsedForm::new(Form::TenQ, false),
                date(2026, 10, 27),
            ),
            filing_ref(
                "1047127",
                "0001047127-26-000052",
                ParsedForm::new(Form::TenQ, false),
                date(2026, 10, 27),
            ),
            filing_ref(
                "109563",
                "0000109563-26-000054",
                ParsedForm::new(Form::TenQ, false),
                date(2026, 10, 27),
            ),
            filing_ref(
                "811156",
                "0000811156-26-000032",
                ParsedForm::new(Form::TenQ, false),
                date(2026, 10, 27),
            ),
            filing_ref(
                "2113605",
                "0001213900-26-112626",
                ParsedForm::new(Form::TwentyF, false),
                date(2026, 10, 27),
            ),
        ];

        let result = computed_filings(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_first_row_when_a_filing_has_two_filers() {
        let index_text = SAMPLE_DAILY_INDEX.to_string();
        let mut state = test_state(Some(index_text.into_bytes()));
        let shared_accession_number = AccessionNumber::new("0000811156-26-000032").expect(
            "Given a hardcoded accession number of the right shape, creation should always succeed",
        );

        let expected_result = vec![filing_ref(
            "811156",
            "0000811156-26-000032",
            ParsedForm::new(Form::TenQ, false),
            date(2026, 10, 27),
        )];

        let result: Vec<FilingRef> = computed_filings(&mut state)
            .await
            .into_iter()
            .filter(|filing| filing.accession_number() == &shared_accession_number)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_drop_the_eight_k_row_for_sample_daily_index() {
        let index_text = SAMPLE_DAILY_INDEX.to_string();
        let eight_k_accession_number = AccessionNumber::new("0001213900-26-112698").expect(
            "Given a hardcoded accession number of the right shape, creation should always succeed",
        );
        let index_holds_eight_k = parse_daily_index(&index_text)
            .expect("Given the real daily index fixture, parsing should always succeed")
            .iter()
            .any(|row| {
                row.accession_number() == &eight_k_accession_number && row.form_text() == "8-K"
            });
        let mut state = test_state(Some(index_text.into_bytes()));

        let expected_result = (true, false, false);

        let filings = computed_filings(&mut state).await;
        let output_holds_eight_k = filings
            .iter()
            .any(|filing| filing.accession_number() == &eight_k_accession_number);
        let result = (
            index_holds_eight_k,
            filings.is_empty(),
            output_holds_eight_k,
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_filing_when_company_name_holds_a_byte_outside_utf8() {
        let row_line = index_line("10-K", "320193", "0000320193-26-000106");
        let (form_column, rest_of_line) = row_line.split_at(17);
        // 0xC9 is the Latin-1 'É', which is not valid UTF-8.
        let index_bytes = [
            INDEX_HEADER.as_bytes(),
            form_column.as_bytes(),
            b"SOCI\xC9T\xC9 ",
            rest_of_line.as_bytes(),
        ]
        .concat();
        let mut state = test_state(Some(index_bytes));

        let expected_result = vec![filing_ref(
            "320193",
            "0000320193-26-000106",
            ParsedForm::new(Form::TenK, false),
            date(2026, 10, 30),
        )];

        let result = computed_filings(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_ten_k_amendment_with_is_amendment_set_when_index_holds_ten_k_a() {
        let index_text = format!(
            "{INDEX_HEADER}{}",
            index_line("10-K/A", "320193", "0000320193-26-000106")
        );
        let mut state = test_state(Some(index_text.into_bytes()));

        let expected_result = vec![filing_ref(
            "320193",
            "0000320193-26-000106",
            ParsedForm::new(Form::TenK, true),
            date(2026, 10, 30),
        )];

        let result = computed_filings(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_forty_f_and_drop_ten_kt_when_index_holds_both() {
        let index_text = format!(
            "{INDEX_HEADER}{}{}",
            index_line("10-KT", "320193", "0000320193-26-000107"),
            index_line("40-F", "1000275", "0001000275-26-000012")
        );
        let mut state = test_state(Some(index_text.into_bytes()));

        let expected_result = vec![filing_ref(
            "1000275",
            "0001000275-26-000012",
            ParsedForm::new(Form::FortyF, false),
            date(2026, 10, 30),
        )];

        let result = computed_filings(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_no_filings_when_index_has_no_rows() {
        let index_text = INDEX_HEADER.to_string();
        let mut state = test_state(Some(index_text.into_bytes()));

        let expected_result: Vec<FilingRef> = Vec::new();

        let result = computed_filings(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_no_filings_when_day_has_no_filings() {
        let mut state = test_state(None);

        let expected_result: Vec<FilingRef> = Vec::new();

        let result = computed_filings(&mut state).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_daily_index_when_index_has_no_header() {
        let index_text = index_line("10-K", "320193", "0000320193-26-000106");
        let mut state = test_state(Some(index_text.into_bytes()));

        let expected_result = StateError::InvalidDailyIndex(InvalidDailyIndex::new(
            STATE_NAME,
            DailyIndexError::MissingDailyIndexHeader,
        ));

        let result = state.compute_output_data_async().await.expect_err(
            "Given an index without a line of dashes, the selection should always fail",
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_daily_index_when_a_row_has_an_invalid_filed_date() {
        let index_text = format!(
            "{INDEX_HEADER}{}",
            index_line("10-K", "320193", "0000320193-26-000106").replace("20261030", "2026103")
        );
        let mut state = test_state(Some(index_text.into_bytes()));

        let expected_result = StateError::InvalidDailyIndex(InvalidDailyIndex::new(
            STATE_NAME,
            DailyIndexError::InvalidDailyIndexLine {
                line_number: 3,
                reason: InvalidDailyIndexLineReason::InvalidFiledDate {
                    filed_date: "2026103".to_string(),
                },
            },
        ));

        let result = state
            .compute_output_data_async()
            .await
            .expect_err("Given a row with a 7-digit date filed, the selection should always fail");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_leave_output_empty_when_index_has_no_header() {
        let index_text = index_line("10-K", "320193", "0000320193-26-000106");
        let mut state = test_state(Some(index_text.into_bytes()));

        let expected_result = None;

        let _ = state.compute_output_data_async().await;
        let result = state.output_data();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = test_state(None);

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_produce_output_when_calling_sync_compute_outside_tokio_runtime() {
        let mut state = test_state(None);

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[test]
    #[should_panic(expected = "compute_output_data failed")]
    fn should_panic_when_calling_sync_compute_with_index_without_header() {
        let index_text = index_line("10-K", "320193", "0000320193-26-000106");
        let mut state = test_state(Some(index_text.into_bytes()));

        state.compute_output_data();
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_select_filings() {
        implements_value_traits::<SelectFilings>();
    }
}
