//! # Filing Reference
//!
//! Provides [`FilingRef`], which names one filing to fetch, and its conversion from a
//! [`DailyIndexRow`].

use std::fmt::{self, Display, Formatter};

use chrono::NaiveDate;
use serde::Serialize;

use crate::shared::cik::Cik;
use crate::shared::daily_index_row::DailyIndexRow;
use crate::shared::financial::accession_number::AccessionNumber;
use crate::shared::financial::form::{Form, ParsedForm, SELECTED_FORMS, UnsupportedForm};

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

impl TryFrom<&DailyIndexRow> for FilingRef {
    type Error = UnsupportedForm;

    /// Converts a row whose form is in [`SELECTED_FORMS`], or an amendment of one.
    ///
    /// # Errors
    ///
    /// Returns an [`UnsupportedForm`] if [`Form::from_sec_str`] rejects the form text of the row,
    /// or if its form is not in [`SELECTED_FORMS`].
    fn try_from(row: &DailyIndexRow) -> Result<Self, Self::Error> {
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const ACCESSION_NUMBER: &str = "0000320193-26-000106";

    fn cik() -> Cik {
        Cik::new("320193").expect("Given a hardcoded valid CIK, parsing should always succeed")
    }

    fn filed_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 27)
            .expect("Given a hardcoded calendar date, the date should always build")
    }

    fn row(form_text: &str) -> DailyIndexRow {
        DailyIndexRow::new(
            form_text.to_string(),
            cik(),
            filed_date(),
            AccessionNumber::new(ACCESSION_NUMBER),
        )
    }

    fn filing_ref(form: Form, is_amendment: bool) -> FilingRef {
        FilingRef::new(
            cik(),
            AccessionNumber::new(ACCESSION_NUMBER),
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
        let daily_index_row = row("10-K");

        let expected_result = Ok(filing_ref(Form::TenK, false));

        let result = FilingRef::try_from(&daily_index_row);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_set_amendment_when_form_is_ten_k_amendment() {
        let daily_index_row = row("10-K/A");

        let expected_result = Ok(filing_ref(Form::TenK, true));

        let result = FilingRef::try_from(&daily_index_row);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_row_for_each_selected_form() {
        let daily_index_rows: Vec<DailyIndexRow> = SELECTED_FORMS
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
        let daily_index_row = row("8-K");

        let expected_result = Err(UnsupportedForm::new("8-K"));

        let result = FilingRef::try_from(&daily_index_row);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unsupported_form_when_form_text_has_two_words() {
        let daily_index_row = row("DEF 14A");

        let expected_result = Err(UnsupportedForm::new("DEF 14A"));

        let result = FilingRef::try_from(&daily_index_row);

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filing_ref() {
        implements_auto_traits::<FilingRef>();
    }
}
