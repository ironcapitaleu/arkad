//! # Period
//!
//! Provides the [`Period`] that a fact covers in the format of the `/v1` contract.
//!
//! ## Modules
//!
//! - [`unsupported_period`]: The [`UnsupportedPeriod`] error for a kind of period that the
//!   contract has no shape for.

use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;
use xbrl::core::period::Period as DomainPeriod;

use crate::traits::from_domain::FromDomain;

pub mod unsupported_period;

pub use unsupported_period::UnsupportedPeriod;

/// The period that a fact covers: `duration` with `start` and `end`, or `instant` with `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Period {
    /// A range of days, for example a fiscal year of an income statement.
    Duration {
        /// The first day, or `null` if the source gives only the number of quarters.
        #[schema(value_type = Option<String>, format = Date, example = "2023-10-01")]
        start: Option<NaiveDate>,
        /// The last day.
        #[schema(value_type = String, format = Date, example = "2024-09-28")]
        end: NaiveDate,
    },
    /// One day, for example a balance sheet date.
    Instant {
        /// The day.
        #[schema(value_type = String, format = Date, example = "2024-09-28")]
        end: NaiveDate,
    },
}

impl TryFrom<DomainPeriod> for Period {
    type Error = UnsupportedPeriod;

    /// Converts an `xbrl` [`Period`](DomainPeriod) into a contract [`Period`].
    ///
    /// # Errors
    ///
    /// Returns an [`UnsupportedPeriod`] for a kind of period other than an instant or a duration.
    fn try_from(period: DomainPeriod) -> Result<Self, Self::Error> {
        match period {
            DomainPeriod::Instant { end } => Ok(Self::Instant { end }),
            DomainPeriod::Duration(duration) => Ok(Self::Duration {
                start: duration.start(),
                end: duration.end(),
            }),
            // The `xbrl` period is `#[non_exhaustive]`, so a new kind needs a contract shape first.
            other => Err(UnsupportedPeriod::new(other.to_string())),
        }
    }
}

impl FromDomain<DomainPeriod> for Period {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use xbrl::core::period::Duration;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_period() {
        implements_auto_traits::<Period>();
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a valid hardcoded date, building it should always succeed")
    }

    #[test]
    fn should_keep_start_and_end_when_the_domain_period_is_a_duration() {
        let duration = Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
            .expect("Given a hardcoded start before the end, the creation should always succeed");
        let domain = DomainPeriod::Duration(duration);

        let expected_result = Ok(Period::Duration {
            start: Some(date(2023, 10, 1)),
            end: date(2024, 9, 28),
        });

        let result = Period::try_from(domain);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_only_kind_and_end_for_instant() {
        let period = Period::Instant {
            end: date(2024, 9, 28),
        };

        let expected_result = json!({ "kind": "instant", "end": "2024-09-28" });

        let result = serde_json::to_value(period)
            .expect("Given an enum of dates, serializing it should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_end_when_the_domain_period_is_an_instant() {
        let domain = DomainPeriod::Instant {
            end: date(2024, 9, 28),
        };

        let expected_result = Ok(Period::Instant {
            end: date(2024, 9, 28),
        });

        let result = Period::try_from(domain);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_have_no_start_when_the_domain_duration_holds_only_quarters() {
        let domain = DomainPeriod::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 4));

        let expected_result = Ok(Period::Duration {
            start: None,
            end: date(2024, 9, 28),
        });

        let result = Period::try_from(domain);

        assert_eq!(result, expected_result);
    }
}
