//! # Filing Source Module
//!
//! Provides the [`FilingSource`] struct for tracking data lineage back to specific SEC filings.
//! Every financial observation carries a `FilingSource` to maintain a complete audit trail
//! of where each data point originated.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use chrono::NaiveDate;

use crate::shared::financial::accession_number::AccessionNumber;
use crate::shared::financial::fiscal_period::FiscalPeriod;
use crate::shared::financial::fiscal_year::FiscalYear;
use crate::shared::financial::form::Form;

/// Provenance metadata for a financial data point.
///
/// Tracks which SEC filing a data point came from, enabling full data lineage
/// from the final financial statement back to the original SEC submission.
///
/// # Example
/// ```
/// use chrono::NaiveDate;
/// use sec::shared::financial::accession_number::AccessionNumber;
/// use sec::shared::financial::filing_source::FilingSource;
/// use sec::shared::financial::fiscal_period::FiscalPeriod;
/// use sec::shared::financial::fiscal_year::FiscalYear;
/// use sec::shared::financial::form::Form;
///
/// let source = FilingSource::new(
///     AccessionNumber::new("0000320193-23-000106"),
///     Form::TenK,
///     FiscalYear::from(2023_u16),
///     FiscalPeriod::Fy,
///     NaiveDate::from_ymd_opt(2023, 11, 3).unwrap(),
///     NaiveDate::from_ymd_opt(2023, 9, 30).unwrap(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct FilingSource {
    accession_number: AccessionNumber,
    form: Form,
    fiscal_year: FiscalYear,
    fiscal_period: FiscalPeriod,
    filed_date: NaiveDate,
    period_end: NaiveDate,
}

impl FilingSource {
    /// Creates a new [`FilingSource`] with the given metadata.
    #[must_use]
    pub const fn new(
        accession_number: AccessionNumber,
        form: Form,
        fiscal_year: FiscalYear,
        fiscal_period: FiscalPeriod,
        filed_date: NaiveDate,
        period_end: NaiveDate,
    ) -> Self {
        Self {
            accession_number,
            form,
            fiscal_year,
            fiscal_period,
            filed_date,
            period_end,
        }
    }

    /// Returns a reference to the accession number.
    #[must_use]
    pub const fn accession_number(&self) -> &AccessionNumber {
        &self.accession_number
    }

    /// Returns the filing form type.
    #[must_use]
    pub const fn form(&self) -> Form {
        self.form
    }

    /// Returns the fiscal year.
    #[must_use]
    pub const fn fiscal_year(&self) -> FiscalYear {
        self.fiscal_year
    }

    /// Returns the fiscal period.
    #[must_use]
    pub const fn fiscal_period(&self) -> FiscalPeriod {
        self.fiscal_period
    }

    /// Returns the date the filing was submitted to the SEC.
    #[must_use]
    pub const fn filed_date(&self) -> NaiveDate {
        self.filed_date
    }

    /// Returns the end date of the reporting period.
    #[must_use]
    pub const fn period_end(&self) -> NaiveDate {
        self.period_end
    }
}

impl Display for FilingSource {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} (filed {})",
            self.accession_number, self.form, self.fiscal_year, self.fiscal_period, self.filed_date
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn create_test_filing_source() -> FilingSource {
        FilingSource::new(
            AccessionNumber::new("0000320193-23-000106"),
            Form::TenK,
            FiscalYear::from(2023_u16),
            FiscalPeriod::Fy,
            NaiveDate::from_ymd_opt(2023, 11, 3).expect("Hardcoded date should always be valid"),
            NaiveDate::from_ymd_opt(2023, 9, 30).expect("Hardcoded date should always be valid"),
        )
    }

    #[test]
    fn should_return_accession_number_when_accessed() {
        let source = create_test_filing_source();

        let expected_result = &AccessionNumber::new("0000320193-23-000106");

        let result = source.accession_number();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_form_when_accessed() {
        let source = create_test_filing_source();

        let expected_result = Form::TenK;

        let result = source.form();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_fiscal_year_when_accessed() {
        let source = create_test_filing_source();

        let expected_result = FiscalYear::from(2023_u16);

        let result = source.fiscal_year();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_filing_source_when_formatted() {
        let source = create_test_filing_source();

        let expected_result = "0000320193-23-000106 10-K 2023 FY (filed 2023-11-03)";

        let result = source.to_string();

        assert_eq!(result, expected_result);
    }
}
