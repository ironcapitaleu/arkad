//! # Period Identity
//!
//! Provides [`derive_q4`], the period identity `core/period/q4-from-fy@1`: the fourth quarter is the
//! fiscal year less the first nine months.

use std::collections::BTreeSet;
use std::str::FromStr;

use jiff::civil::Date;

use super::candidate::SeriesKey;
use super::confidence::Confidence;
use super::dedup::Series;
use super::period::{Duration, Period};
use super::rule::RuleId;
use super::standardized::StandardizedFact;

/// The ID of the rule that derives the fourth quarter from the fiscal year.
pub const Q4_FROM_FY: &str = "core/period/q4-from-fy@1";

/// The length of a fiscal year, in quarters.
const YEAR_LENGTH: u16 = 4;

/// The length of the year to date at the end of the third quarter, in quarters.
const NINE_MONTHS_LENGTH: u16 = 3;

/// The length of the derived fourth quarter, in quarters.
const QUARTER_LENGTH: u16 = 1;

/// The days from start to end under which a fiscal year is short: 51 weeks. A 52-53 week year
/// spans 363 or 370 days, and a calendar year 364 or 365.
const SHORT_YEAR_DAYS: i64 = 357;

/// Applies `core/period/q4-from-fy@1` to the series of one entity after
/// `core/dedup/newest-filing@1`, and returns the derived facts.
///
/// For each fiscal year value, the rule looks for the nine-month value of the same concept and
/// unit, whose end lies one quarter before the end of the year. The year must end 47 to
/// 137 days after the nine-month end, so that the fourth quarter rounds to one quarter. If
/// several nine-month values fit, the rule takes the latest end. The two periods must have the same
/// exact start. The fourth quarter is the year less the nine months. It runs
/// from the day after the nine-month end to the year end.
///
/// The rule reads only fiscal year values without dimensions, in one currency, such as `USD`.
/// Per-share values, share counts, and ratios do not add over periods. The rule derives nothing for a fourth quarter that the
/// series already holds or that a conflict blocks.
///
/// The rule derives nothing from a short year, a year that spans under 357 days, because its fourth
/// quarter is not the year less nine months. A first fiscal year from inception and a transition
/// period are such years.
///
/// The confidence is [`Confidence::Derived`], or the weaker confidence of an input. The fact has
/// `restated` if an input has it. It carries the release of the fiscal year value. It carries the
/// scope of the inputs if both carry the same scope, and no scope otherwise.
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
        .filter(|year| {
            year.dimensions.is_empty()
                && year.unit.is_currency()
                && year.period.quarters() == YEAR_LENGTH
        })
        .filter(|year| !is_short_year(year))
        .filter_map(|year| {
            let mut key = year.series_key();
            key.quarters = QUARTER_LENGTH;
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

/// Returns `true` if `year` spans under 357 days.
fn is_short_year(year: &StandardizedFact) -> bool {
    match year.period {
        Period::Duration(period) => is_short_span(period.start(), period.end()),
        Period::Instant { .. } => false,
    }
}

/// Returns `true` if the span from `start` to `end` lasts under 357 days.
fn is_short_span(start: Date, end: Date) -> bool {
    i64::from((end - start).get_days()) < SHORT_YEAR_DAYS
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
    let same_start = year_period.start() == nine_months_period.start();
    if !same_series || !same_start || nine_months_period.quarters() != NINE_MONTHS_LENGTH {
        return None;
    }
    let start = nine_months_period.end().tomorrow().ok()?;
    let quarter = Duration::from_dates(start, year_period.end()).ok()?;
    (quarter.quarters() == QUARTER_LENGTH).then_some((quarter, nine_months))
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
        scope: year.scope.filter(|scope| nine_months.scope == Some(*scope)),
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
    use jiff::civil::Date;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::IncomeStatementConcept;
    use crate::core::fact::Dimensions;
    use crate::core::scope::Scope;
    use crate::core::unit::Unit;
    use crate::tests::fixtures::sample_fact::{date, in_cloud_segment, standardized};

    fn span(start: Date, end: Date) -> Period {
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
    fn should_derive_nothing_when_the_year_is_a_short_year_of_eleven_months() {
        let facts = series(vec![
            standardized(span(date(2023, 2, 1), date(2023, 12, 31)), "330000000"),
            standardized(span(date(2023, 2, 1), date(2023, 10, 31)), "270000000"),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_the_year_has_53_weeks() {
        let facts = series(vec![
            standardized(span(date(2022, 9, 25), date(2023, 9, 30)), "383285000000"),
            standardized(span(date(2022, 9, 25), date(2023, 7, 1)), "293787000000"),
        ]);

        let expected_result = vec![BigDecimal::from(89_498_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_the_year_spans_357_days() {
        let facts = series(vec![
            standardized(span(date(2023, 1, 1), date(2023, 12, 24)), "400000000"),
            standardized(span(date(2023, 1, 1), date(2023, 9, 30)), "300000000"),
        ]);

        let expected_result = vec![BigDecimal::from(100_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_year_spans_356_days() {
        let facts = series(vec![
            standardized(span(date(2023, 1, 1), date(2023, 12, 23)), "400000000"),
            standardized(span(date(2023, 1, 1), date(2023, 9, 30)), "300000000"),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

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
    fn should_carry_the_scope_of_the_inputs_when_both_carry_the_same_scope() {
        let mut year = standardized(fiscal_year(), "391035000000");
        year.scope = Some(Scope::ExcludesRestrictedCash);
        let mut first_nine_months = standardized(nine_months(), "296105000000");
        first_nine_months.scope = Some(Scope::ExcludesRestrictedCash);
        let facts = series(vec![year, first_nine_months]);

        let expected_result = vec![Some(Scope::ExcludesRestrictedCash)];

        let result: Vec<Option<Scope>> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.scope)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_carry_no_scope_when_only_the_year_carries_a_scope() {
        let mut year = standardized(fiscal_year(), "391035000000");
        year.scope = Some(Scope::ExcludesRestrictedCash);
        let first_nine_months = standardized(nine_months(), "296105000000");
        let facts = series(vec![year, first_nine_months]);

        let expected_result = vec![None];

        let result: Vec<Option<Scope>> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.scope)
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

    #[test]
    fn should_derive_nothing_when_the_nine_months_are_another_concept() {
        let mut first_nine_months = standardized(nine_months(), "296105000000");
        first_nine_months.concept = IncomeStatementConcept::CostOfRevenue.element();
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            first_nine_months,
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_nine_months_have_another_unit() {
        let mut first_nine_months = standardized(nine_months(), "296105000000");
        first_nine_months.unit = Unit::from_str("EUR")
            .expect("Given a hardcoded unit with one measure, the parse should always succeed");
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            first_nine_months,
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_fiscal_year_value_carries_a_business_segment() {
        let facts = series(in_cloud_segment(vec![
            standardized(fiscal_year(), "391035000000"),
            standardized(nine_months(), "296105000000"),
        ]));

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_nine_months_have_other_dimensions() {
        let mut first_nine_months = standardized(nine_months(), "296105000000");
        first_nine_months.dimensions =
            Dimensions::new([("StatementGeographicalAxis", "AmericasSegmentMember")])
                .expect("Given one hardcoded axis and member, the creation should always succeed");
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            first_nine_months,
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_latest_nine_month_end_when_two_nine_month_values_fit() {
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            standardized(span(date(2023, 10, 1), date(2024, 6, 1)), "250000000000"),
            standardized(nine_months(), "296105000000"),
        ]);

        let expected_result = vec![BigDecimal::from(94_930_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_mark_the_fourth_quarter_restated_when_the_year_is_restated() {
        let mut year = standardized(fiscal_year(), "391035000000");
        year.restated = true;
        let facts = series(vec![year, standardized(nine_months(), "296105000000")]);

        let expected_result = vec![true];

        let result: Vec<bool> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.restated)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_weaker_confidence_when_the_nine_months_are_computed() {
        let mut first_nine_months = standardized(nine_months(), "296105000000");
        first_nine_months.confidence = Confidence::Computed;
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            first_nine_months,
        ]);

        let expected_result = vec![Confidence::Computed];

        let result: Vec<Confidence> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.confidence)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_the_year_ends_137_days_after_the_nine_months() {
        let facts = series(vec![
            standardized(span(date(2023, 9, 25), date(2024, 9, 28)), "391035000000"),
            standardized(span(date(2023, 9, 25), date(2024, 5, 14)), "296105000000"),
        ]);

        let expected_result = vec![BigDecimal::from(94_930_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts)
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_year_ends_138_days_after_the_nine_months() {
        let facts = series(vec![
            standardized(span(date(2023, 9, 25), date(2024, 9, 28)), "391035000000"),
            standardized(span(date(2023, 9, 25), date(2024, 5, 13)), "296105000000"),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts);

        assert_eq!(result, expected_result);
    }
}
