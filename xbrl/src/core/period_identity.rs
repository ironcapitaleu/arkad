//! # Period Identity
//!
//! Provides [`derive_q4`], the period identity `core/period/q4-from-fy@1`: the fourth quarter is the
//! fiscal year less the first nine months.

use std::collections::BTreeSet;
use std::str::FromStr;

use super::candidate::SeriesKey;
use super::confidence::Confidence;
use super::dedup::Series;
use super::period::{Duration, Period};
use super::rule::RuleId;
use super::standardized::StandardizedFact;

/// The ID of the rule that derives the fourth quarter from the fiscal year.
pub const Q4_FROM_FY: &str = "core/period/q4-from-fy@1";

/// The length in quarters of a fiscal year.
const YEAR_QUARTERS: u16 = 4;

/// The length in quarters of the year to date at the end of the third quarter.
const NINE_MONTHS_QUARTERS: u16 = 3;

/// The length in quarters of the derived fourth quarter.
const QUARTER: u16 = 1;

/// Applies `core/period/q4-from-fy@1` to the series of one entity after
/// `core/dedup/newest-filing@1`, and returns the derived facts.
///
/// For each fiscal year value, the rule looks for the nine-month value of the same concept, unit,
/// and dimensions, whose end lies one quarter before the end of the year. If both periods have an
/// exact start, the starts must be equal. The fourth quarter is the year less the nine months. It
/// runs from the day after the nine-month end to the year end.
///
/// The rule derives only values in one currency, such as `USD`. Per-share values, share counts,
/// and ratios do not add over periods. It derives nothing for a fourth quarter that the series
/// already holds or that a conflict blocks.
///
/// The confidence is [`Confidence::Derived`], or the weaker confidence of an input. The fact has
/// `restated` if an input has it. It carries the release of the fiscal year value.
#[must_use]
pub fn derive_q4(series: &Series) -> Vec<StandardizedFact> {
    let known: BTreeSet<SeriesKey> = series
        .facts
        .iter()
        .map(StandardizedFact::series_key)
        .chain(series.blocked.iter().cloned())
        .collect();
    series
        .facts
        .iter()
        .filter(|year| year.unit.is_currency() && length_of(year) == Some(YEAR_QUARTERS))
        .filter_map(|year| {
            let mut key = year.series_key();
            key.quarters = QUARTER;
            if known.contains(&key) {
                return None;
            }
            series
                .facts
                .iter()
                .filter_map(|nine_months| fourth_quarter(year, nine_months))
                .max_by_key(|(quarter, _)| quarter.start())
                .map(|(quarter, nine_months)| subtract(year, nine_months, quarter))
        })
        .collect()
}

/// Returns the length in quarters of a duration, or `None` for an instant.
const fn length_of(fact: &StandardizedFact) -> Option<u16> {
    match fact.period {
        Period::Duration(duration) => Some(duration.quarters()),
        _ => None,
    }
}

/// Returns the fourth quarter between `nine_months` and `year`, or `None` if `nine_months` is not
/// the first nine months of `year`.
fn fourth_quarter<'a>(
    year: &StandardizedFact,
    nine_months: &'a StandardizedFact,
) -> Option<(Duration, &'a StandardizedFact)> {
    let (Period::Duration(year_period), Period::Duration(nine_months_period)) =
        (year.period, nine_months.period)
    else {
        return None;
    };
    let same_series = nine_months.entity == year.entity
        && nine_months.concept == year.concept
        && nine_months.unit == year.unit
        && nine_months.dimensions == year.dimensions;
    let same_start = match (year_period.start(), nine_months_period.start()) {
        (Some(year_start), Some(nine_months_start)) => year_start == nine_months_start,
        _ => true,
    };
    if !same_series || !same_start || nine_months_period.quarters() != NINE_MONTHS_QUARTERS {
        return None;
    }
    let start = nine_months_period.end().succ_opt()?;
    let quarter = Duration::from_dates(start, year_period.end()).ok()?;
    (quarter.quarters() == QUARTER).then_some((quarter, nine_months))
}

fn subtract(
    year: &StandardizedFact,
    nine_months: &StandardizedFact,
    quarter: Duration,
) -> StandardizedFact {
    let mut inputs = year.inputs.clone();
    inputs.extend(nine_months.inputs.iter().cloned());
    StandardizedFact {
        entity: year.entity.clone(),
        concept: year.concept,
        period: Period::Duration(quarter),
        unit: year.unit.clone(),
        dimensions: year.dimensions.clone(),
        value: &year.value - &nine_months.value,
        confidence: Confidence::Derived
            .max(year.confidence)
            .max(nine_months.confidence),
        rule: q4_rule(),
        release: year.release.clone(),
        inputs,
        restated: year.restated || nine_months.restated,
    }
}

fn q4_rule() -> RuleId {
    RuleId::from_str(Q4_FROM_FY).expect(
        "Given the hardcoded rule ID in the documented form, the parse should always succeed",
    )
}

#[cfg(test)]
mod tests {
    use bigdecimal::BigDecimal;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::unit::Unit;
    use crate::tests::fixtures::sample_fact::{date, standardized};

    fn span(start: NaiveDate, end: NaiveDate) -> Period {
        Period::Duration(
            Duration::from_dates(start, end)
                .expect("Given an end after the start, the creation should always succeed"),
        )
    }

    fn fiscal_year() -> Period {
        span(date(2023, 10, 1), date(2024, 9, 28))
    }

    fn nine_months() -> Period {
        span(date(2023, 10, 1), date(2024, 6, 29))
    }

    fn fourth_quarter() -> Period {
        span(date(2024, 6, 30), date(2024, 9, 28))
    }

    fn series(facts: Vec<StandardizedFact>) -> Series {
        Series {
            facts,
            blocked: Vec::new(),
        }
    }

    fn summary(facts: Vec<StandardizedFact>) -> Vec<(Period, BigDecimal, Confidence, String)> {
        facts
            .into_iter()
            .map(|fact| {
                (
                    fact.period,
                    fact.value,
                    fact.confidence,
                    fact.rule.to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_the_year_and_nine_months_are_known() {
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            standardized(nine_months(), "296105000000"),
        ]);

        let expected_result = vec![(
            fourth_quarter(),
            BigDecimal::from(94_930_000_000_i64),
            Confidence::Derived,
            "core/period/q4-from-fy@1".to_string(),
        )];

        let result = summary(derive_q4(&facts));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_the_periods_have_no_exact_start() {
        let facts = series(vec![
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 4)),
                "391035000000",
            ),
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 6, 29), 3)),
                "296105000000",
            ),
        ]);

        let expected_result = vec![(
            fourth_quarter(),
            BigDecimal::from(94_930_000_000_i64),
            Confidence::Derived,
            "core/period/q4-from-fy@1".to_string(),
        )];

        let result = summary(derive_q4(&facts));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_join_the_inputs_of_the_year_and_the_nine_months_when_deriving() {
        let year = standardized(fiscal_year(), "391035000000");
        let first_nine_months = standardized(nine_months(), "296105000000");
        let year_key = year.inputs[0].clone();
        let nine_months_key = first_nine_months.inputs[0].clone();
        let facts = series(vec![year, first_nine_months]);

        let expected_result = vec![vec![year_key, nine_months_key]];

        let result: Vec<_> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.inputs)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_unit_is_per_share() {
        let mut year = standardized(fiscal_year(), "6.11");
        let mut first_nine_months = standardized(nine_months(), "4.47");
        let per_share = Unit::from_str("USD/shares").expect(
            "Given a hardcoded unit with one measure per side, the parse should always succeed",
        );
        year.unit = per_share.clone();
        first_nine_months.unit = per_share;
        let facts = series(vec![year, first_nine_months]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_unit_is_shares() {
        let mut year = standardized(fiscal_year(), "15000000000");
        let mut first_nine_months = standardized(nine_months(), "15100000000");
        let shares = Unit::from_str("shares")
            .expect("Given a hardcoded unit with one measure, the parse should always succeed");
        year.unit = shares.clone();
        first_nine_months.unit = shares;
        let facts = series(vec![year, first_nine_months]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_mark_the_fourth_quarter_restated_when_an_input_is_restated() {
        let year = standardized(fiscal_year(), "391035000000");
        let mut first_nine_months = standardized(nine_months(), "296105000000");
        first_nine_months.restated = true;
        let facts = series(vec![year, first_nine_months]);

        let expected_result = vec![true];

        let result: Vec<bool> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.restated)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_weaker_confidence_when_an_input_is_computed() {
        let mut year = standardized(fiscal_year(), "391035000000");
        year.confidence = Confidence::Computed;
        let first_nine_months = standardized(nine_months(), "296105000000");
        let facts = series(vec![year, first_nine_months]);

        let expected_result = vec![Confidence::Computed];

        let result: Vec<Confidence> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.confidence)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_series_holds_the_fourth_quarter() {
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            standardized(nine_months(), "296105000000"),
            standardized(fourth_quarter(), "94930000000"),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_a_conflict_blocks_the_fourth_quarter() {
        let blocked = standardized(fourth_quarter(), "94930000000").series_key();
        let facts = Series {
            facts: vec![
                standardized(fiscal_year(), "391035000000"),
                standardized(nine_months(), "296105000000"),
            ],
            blocked: vec![blocked],
        };

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_nine_months_start_in_another_fiscal_year() {
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            standardized(span(date(2023, 9, 24), date(2024, 6, 29)), "296105000000"),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_year_is_an_instant() {
        let facts = series(vec![
            standardized(
                Period::Instant {
                    end: date(2024, 9, 28),
                },
                "391035000000",
            ),
            standardized(nine_months(), "296105000000"),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }
}
