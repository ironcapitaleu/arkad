//! # Provenance
//!
//! Filing metadata that tracks where a data point came from.

use chrono::NaiveDate;

use super::accession_number::AccessionNumber;
use super::fiscal_period::FiscalPeriod;
use super::fiscal_year::FiscalYear;
use super::form::Form;

/// Provenance metadata identifying the SEC filing a data point originates from.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Provenance {
    /// The unique filing identifier.
    accession_number: AccessionNumber,
    /// The filing form type.
    form: Form,
    /// The fiscal year the filing covers.
    fiscal_year: FiscalYear,
    /// The fiscal period within the year.
    fiscal_period: FiscalPeriod,
    /// The date the filing was submitted to the SEC.
    filed_date: NaiveDate,
    /// The end date of the reported period.
    period_end: NaiveDate,
}

impl Provenance {
    /// Creates a new [`Provenance`] from its components.
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

    /// Returns the accession number.
    #[must_use]
    pub const fn accession_number(&self) -> &AccessionNumber {
        &self.accession_number
    }

    /// Returns the form type.
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

    /// Returns the filing date.
    #[must_use]
    pub const fn filed_date(&self) -> NaiveDate {
        self.filed_date
    }

    /// Returns the period end date.
    #[must_use]
    pub const fn period_end(&self) -> NaiveDate {
        self.period_end
    }
}
