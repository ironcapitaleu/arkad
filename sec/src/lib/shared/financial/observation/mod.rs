//! # Observation Module
//!
//! Provides the [`Observation`] struct representing a single measured financial data point
//! with full typing and lineage. Each observation carries its value, unit, time period,
//! optional SEC frame identifier, and the filing it originated from.

use std::fmt;

use crate::shared::financial::filing_source::FilingSource;
use crate::shared::financial::frame::Frame;
use crate::shared::financial::period::Period;
use crate::shared::financial::unit::Unit;

/// A single measured financial data point with full typing and lineage.
///
/// Represents one value in a time series for a financial concept (e.g., "Revenue was $383B USD
/// for the period Oct 2022 - Sep 2023, from filing 10-K FY2023").
///
/// # Example
/// ```
/// use chrono::NaiveDate;
/// use sec::shared::financial::accession_number::AccessionNumber;
/// use sec::shared::financial::filing_source::FilingSource;
/// use sec::shared::financial::fiscal_period::FiscalPeriod;
/// use sec::shared::financial::fiscal_year::FiscalYear;
/// use sec::shared::financial::form::Form;
/// use sec::shared::financial::frame::Frame;
/// use sec::shared::financial::observation::Observation;
/// use sec::shared::financial::period::Period;
/// use sec::shared::financial::unit::Unit;
///
/// let obs = Observation::new(
///     383_285_000_000,
///     Unit::Usd,
///     Period::Duration {
///         start: NaiveDate::from_ymd_opt(2022, 10, 1).unwrap(),
///         end: NaiveDate::from_ymd_opt(2023, 9, 30).unwrap(),
///     },
///     Some(Frame::new(2023, None, false)),
///     FilingSource::new(
///         AccessionNumber::new("0000320193-23-000106"),
///         Form::TenK,
///         FiscalYear::from(2023_u16),
///         FiscalPeriod::Fy,
///         NaiveDate::from_ymd_opt(2023, 11, 3).unwrap(),
///         NaiveDate::from_ymd_opt(2023, 9, 30).unwrap(),
///     ),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Observation {
    value: i64,
    unit: Unit,
    period: Period,
    frame: Option<Frame>,
    filing: FilingSource,
}

impl Observation {
    /// Creates a new [`Observation`] with the given components.
    #[must_use]
    pub const fn new(
        value: i64,
        unit: Unit,
        period: Period,
        frame: Option<Frame>,
        filing: FilingSource,
    ) -> Self {
        Self {
            value,
            unit,
            period,
            frame,
            filing,
        }
    }

    /// Returns the observed value.
    #[must_use]
    pub const fn value(&self) -> i64 {
        self.value
    }

    /// Returns the measurement unit.
    #[must_use]
    pub const fn unit(&self) -> Unit {
        self.unit
    }

    /// Returns the time period of this observation.
    #[must_use]
    pub const fn period(&self) -> &Period {
        &self.period
    }

    /// Returns the optional SEC frame identifier.
    #[must_use]
    pub const fn frame(&self) -> Option<&Frame> {
        self.frame.as_ref()
    }

    /// Returns a reference to the filing source (data lineage).
    #[must_use]
    pub const fn filing(&self) -> &FilingSource {
        &self.filing
    }
}

impl fmt::Display for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} [{}] from {}",
            self.value, self.unit, self.period, self.filing
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::fiscal_period::FiscalPeriod;
    use crate::shared::financial::fiscal_year::FiscalYear;
    use crate::shared::financial::form::Form;

    fn create_test_observation() -> Observation {
        Observation::new(
            383_285_000_000,
            Unit::Usd,
            Period::Duration {
                start: NaiveDate::from_ymd_opt(2022, 10, 1)
                    .expect("Hardcoded date should always be valid"),
                end: NaiveDate::from_ymd_opt(2023, 9, 30)
                    .expect("Hardcoded date should always be valid"),
            },
            Some(Frame::new(2023, None, false)),
            FilingSource::new(
                AccessionNumber::new("0000320193-23-000106"),
                Form::TenK,
                FiscalYear::from(2023_u16),
                FiscalPeriod::Fy,
                NaiveDate::from_ymd_opt(2023, 11, 3)
                    .expect("Hardcoded date should always be valid"),
                NaiveDate::from_ymd_opt(2023, 9, 30)
                    .expect("Hardcoded date should always be valid"),
            ),
        )
    }

    #[test]
    fn should_return_value_when_accessed() {
        let obs = create_test_observation();

        let expected_result = 383_285_000_000;

        let result = obs.value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unit_when_accessed() {
        let obs = create_test_observation();

        let expected_result = Unit::Usd;

        let result = obs.unit();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_filing_source_when_accessed() {
        let obs = create_test_observation();

        let expected_result = Form::TenK;

        let result = obs.filing().form();

        assert_eq!(result, expected_result);
    }
}
