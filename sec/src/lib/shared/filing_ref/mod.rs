//! # Filing Reference
//!
//! Provides [`FilingRef`], which names one filing to fetch, and its conversion from a
//! [`FormIndexRow`]. [`select_filings`] and [`select_filings_filed_in`] pick the filings to fetch
//! from the rows of an index.

use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::ops::RangeInclusive;

use chrono::NaiveDate;
use serde::Serialize;

use crate::shared::cik::Cik;
use crate::shared::financial::accession_number::AccessionNumber;
use crate::shared::financial::form::{Form, ParsedForm, SELECTED_FORMS, UnsupportedForm};
use crate::shared::form_index_row::FormIndexRow;

/// The filer, the accession number, the form, and the date filed of one filing to fetch.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FilingRef {
    cik: Cik,
    accession_number: AccessionNumber,
    form: Form,
    is_amendment: bool,
    filed_date: NaiveDate,
}

impl FilingRef {
    /// Creates a new [`FilingRef`]. Takes the form and the amendment flag from `parsed_form`.
    #[must_use]
    pub const fn new(
        cik: Cik,
        accession_number: AccessionNumber,
        parsed_form: ParsedForm,
        filed_date: NaiveDate,
    ) -> Self {
        Self {
            cik,
            accession_number,
            form: parsed_form.form(),
            is_amendment: parsed_form.is_amendment(),
            filed_date,
        }
    }

    /// Returns the CIK of the filer.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the accession number of the filing.
    #[must_use]
    pub const fn accession_number(&self) -> &AccessionNumber {
        &self.accession_number
    }

    /// Returns the base form, for example [`Form::TenK`] for `10-K/A`.
    #[must_use]
    pub const fn form(&self) -> Form {
        self.form
    }

    /// Returns `true` if the filing amends an earlier one.
    #[must_use]
    pub const fn is_amendment(&self) -> bool {
        self.is_amendment
    }

    /// Returns the date filed.
    #[must_use]
    pub const fn filed_date(&self) -> NaiveDate {
        self.filed_date
    }
}

impl Display for FilingRef {
    /// Writes the accession number, which names the filing on its own.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.accession_number)
    }
}

impl TryFrom<&FormIndexRow> for FilingRef {
    type Error = UnsupportedForm;

    /// Converts a row whose form is in [`SELECTED_FORMS`], or an amendment of one.
    ///
    /// # Errors
    ///
    /// Returns an [`UnsupportedForm`] if [`Form::from_sec_str`] rejects the form text of the row,
    /// or if its form is not in [`SELECTED_FORMS`].
    fn try_from(row: &FormIndexRow) -> Result<Self, Self::Error> {
        let parsed_form = Form::from_sec_str(row.form_text())?;
        if !SELECTED_FORMS.contains(&parsed_form.form()) {
            return Err(UnsupportedForm::new(row.form_text()));
        }

        Ok(Self::new(
            row.cik().clone(),
            row.accession_number().clone(),
            parsed_form,
            row.filed_date(),
        ))
    }
}

/// Returns the filings to fetch from the rows of an index: one [`FilingRef`] per accession
/// number, in row order.
///
/// Keeps the rows that [`FilingRef::try_from`] accepts. A filing with several filers has one row
/// per filer, and the first one stays.
#[must_use]
pub fn select_filings<'a>(rows: impl IntoIterator<Item = &'a FormIndexRow>) -> Vec<FilingRef> {
    let mut seen_accession_numbers = HashSet::new();
    rows.into_iter()
        .filter_map(|row| FilingRef::try_from(row).ok())
        .filter(|filing| seen_accession_numbers.insert(filing.accession_number().clone()))
        .collect()
}

/// Returns the filings of [`select_filings`] for the rows with a date filed in `dates`.
///
/// A full index holds every row of its quarter, so a run that reads a range of dates drops the
/// other rows first.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use sec::shared::form_index_row::parse_form_index;
/// use sec::shared::filing_ref::select_filings_filed_in;
///
/// let text = "Form Type   Company Name\n---\n\
///     10-K             APPLE INC                                                     320193      2026-10-30  edgar/data/320193/0000320193-26-000106.txt\n\
///     10-Q             CARMAX INC                                                    1170010     2026-09-30  edgar/data/1170010/0001170010-26-000104.txt";
/// let rows = parse_form_index(text)
///     .expect("Given two hardcoded well-formed rows, parsing should always succeed");
/// let october = NaiveDate::from_ymd_opt(2026, 10, 1)
///     .expect("Given a hardcoded calendar date, the date should always build")
///     ..=NaiveDate::from_ymd_opt(2026, 10, 31)
///         .expect("Given a hardcoded calendar date, the date should always build");
///
/// let expected_result = vec!["0000320193-26-000106".to_string()];
///
/// let result: Vec<String> = select_filings_filed_in(&rows, &october)
///     .iter()
///     .map(ToString::to_string)
///     .collect();
///
/// assert_eq!(result, expected_result);
/// ```
#[must_use]
pub fn select_filings_filed_in<'a>(
    rows: impl IntoIterator<Item = &'a FormIndexRow>,
    dates: &RangeInclusive<NaiveDate>,
) -> Vec<FilingRef> {
    select_filings(
        rows.into_iter()
            .filter(|row| dates.contains(&row.filed_date())),
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::form_index_row::parse_form_index;
    use crate::tests::fixtures::data::daily_index::{
        SAMPLE_DAILY_INDEX, SAMPLE_DAILY_INDEX_2026_09_30, SAMPLE_DAILY_INDEX_2026_10_01,
    };
    use crate::tests::fixtures::data::full_index::{
        SAMPLE_FULL_INDEX_2026_Q3, SAMPLE_FULL_INDEX_2026_Q4,
    };

    const ACCESSION_NUMBER: &str = "0000320193-26-000106";

    fn cik() -> Cik {
        Cik::new("320193").expect("Given a hardcoded valid CIK, parsing should always succeed")
    }

    fn filed_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 27)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    fn row(form_text: &str) -> FormIndexRow {
        FormIndexRow::new(
            form_text.to_string(),
            cik(),
            filed_date(),
            AccessionNumber::new(ACCESSION_NUMBER).expect("Given a hardcoded accession number of the right shape, creation should always succeed"),
        )
    }

    fn filing_ref(form: Form, is_amendment: bool) -> FilingRef {
        FilingRef::new(
            cik(),
            AccessionNumber::new(ACCESSION_NUMBER).expect("Given a hardcoded accession number of the right shape, creation should always succeed"),
            ParsedForm::new(form, is_amendment),
            filed_date(),
        )
    }

    #[test]
    fn should_display_the_accession_number_for_filing_ref() {
        let filing = filing_ref(Form::TenK, false);

        let expected_result = "0000320193-26-000106";

        let result = filing.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_row_when_form_is_ten_k() {
        let form_index_row = row("10-K");

        let expected_result = Ok(filing_ref(Form::TenK, false));

        let result = FilingRef::try_from(&form_index_row);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_set_amendment_when_form_is_ten_k_amendment() {
        let form_index_row = row("10-K/A");

        let expected_result = Ok(filing_ref(Form::TenK, true));

        let result = FilingRef::try_from(&form_index_row);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_row_for_each_selected_form() {
        let daily_index_rows: Vec<FormIndexRow> = SELECTED_FORMS
            .iter()
            .map(|form| row(&form.to_string()))
            .collect();

        let expected_result: Vec<Result<FilingRef, UnsupportedForm>> = vec![
            Ok(filing_ref(Form::TenK, false)),
            Ok(filing_ref(Form::TenQ, false)),
            Ok(filing_ref(Form::TwentyF, false)),
            Ok(filing_ref(Form::FortyF, false)),
        ];

        let result: Vec<Result<FilingRef, UnsupportedForm>> =
            daily_index_rows.iter().map(FilingRef::try_from).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unsupported_form_when_form_is_eight_k() {
        let form_index_row = row("8-K");

        let expected_result = Err(UnsupportedForm::new("8-K"));

        let result = FilingRef::try_from(&form_index_row);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unsupported_form_when_form_text_has_two_words() {
        let form_index_row = row("DEF 14A");

        let expected_result = Err(UnsupportedForm::new("DEF 14A"));

        let result = FilingRef::try_from(&form_index_row);

        assert_eq!(result, expected_result);
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    fn rows_of(texts: &[&str]) -> Vec<FormIndexRow> {
        texts
            .iter()
            .flat_map(|text| {
                parse_form_index(text).expect(
                    "Given the stored fixture of a real index, parsing should always succeed",
                )
            })
            .collect()
    }

    fn filing(
        cik: &str,
        accession_number: &str,
        form: Form,
        is_amendment: bool,
        filed_date: NaiveDate,
    ) -> FilingRef {
        FilingRef::new(
            Cik::new(cik).expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new(accession_number).expect("Given a hardcoded accession number of the right shape, creation should always succeed"),
            ParsedForm::new(form, is_amendment),
            filed_date,
        )
    }

    /// Sorts the filings by accession number, so two indexes with another row order compare.
    fn by_accession_number(mut filings: Vec<FilingRef>) -> Vec<FilingRef> {
        filings.sort_by_key(|filing| filing.accession_number().to_string());
        filings
    }

    fn accession_numbers(filings: &[FilingRef]) -> Vec<String> {
        filings.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn should_keep_one_filing_per_accession_number_in_row_order_for_sample_daily_index() {
        let rows = rows_of(&[SAMPLE_DAILY_INDEX]);

        let expected_result = vec![
            "0001702744-26-000031".to_string(),
            "0001053507-26-000157".to_string(),
            "0001047127-26-000052".to_string(),
            "0000109563-26-000054".to_string(),
            "0000811156-26-000032".to_string(),
            "0001213900-26-112626".to_string(),
        ];

        let result = accession_numbers(&select_filings(&rows));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_the_row_of_the_day_before_when_the_range_starts_on_the_last_day_of_the_quarter()
    {
        let rows = rows_of(&[SAMPLE_FULL_INDEX_2026_Q3]);
        let dates = date(2026, 9, 30)..=date(2026, 9, 30);

        let expected_result = vec![
            "0001493152-26-044953".to_string(),
            "0001193125-26-409138".to_string(),
            "0001170010-26-000104".to_string(),
            "0001104659-26-112415".to_string(),
        ];

        let result = accession_numbers(&select_filings_filed_in(&rows, &dates));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_select_no_filing_when_no_row_has_a_date_filed_in_the_range() {
        let rows = rows_of(&[SAMPLE_FULL_INDEX_2026_Q3]);
        let dates = date(2026, 10, 1)..=date(2026, 10, 31);

        let expected_result: Vec<FilingRef> = Vec::new();

        let result = select_filings_filed_in(&rows, &dates);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_select_the_same_filings_from_the_full_indexes_as_from_the_daily_indexes_when_the_range_crosses_a_quarter_boundary()
     {
        let full_index_rows = rows_of(&[SAMPLE_FULL_INDEX_2026_Q3, SAMPLE_FULL_INDEX_2026_Q4]);
        let daily_index_rows =
            rows_of(&[SAMPLE_DAILY_INDEX_2026_09_30, SAMPLE_DAILY_INDEX_2026_10_01]);
        let dates = date(2026, 9, 30)..=date(2026, 10, 1);

        let filings = vec![
            filing(
                "63754",
                "0000063754-26-000329",
                Form::TenQ,
                false,
                date(2026, 10, 1),
            ),
            filing(
                "1442620",
                "0001104659-26-112415",
                Form::TwentyF,
                false,
                date(2026, 9, 30),
            ),
            filing(
                "1170010",
                "0001170010-26-000104",
                Form::TenQ,
                false,
                date(2026, 9, 30),
            ),
            filing(
                "5981",
                "0001193125-26-409138",
                Form::TenK,
                true,
                date(2026, 9, 30),
            ),
            filing(
                "1883984",
                "0001437749-26-031727",
                Form::TenK,
                true,
                date(2026, 10, 1),
            ),
            filing(
                "1973056",
                "0001493152-26-044953",
                Form::TenK,
                false,
                date(2026, 9, 30),
            ),
            filing(
                "1707303",
                "0001493152-26-045181",
                Form::TwentyF,
                false,
                date(2026, 10, 1),
            ),
        ];
        let expected_result = (filings.clone(), filings);

        let result = (
            by_accession_number(select_filings_filed_in(&full_index_rows, &dates)),
            by_accession_number(select_filings(&daily_index_rows)),
        );

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filing_ref() {
        implements_auto_traits::<FilingRef>();
    }
}
