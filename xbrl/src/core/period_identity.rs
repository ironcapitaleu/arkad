//! # Period Identity
//!
//! Provides [`derive_q4`], the period identity `core/period/q4-from-fy@1`: the fourth quarter is the
//! fiscal year less the first nine months.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use chrono::NaiveDate;

use super::candidate::SeriesKey;
use super::chronology::{changes, is_form};
use super::confidence::Confidence;
use super::dedup::Series;
use super::filing::{EntityId, Filing};
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

/// The days from start to end under which a fiscal year with an exact start is short: 51 weeks.
/// A 52-53 week year spans 363 or 370 days, and a calendar year 364 or 365.
const SHORT_YEAR_DAYS: i64 = 357;

/// The form of a transition report. It covers the short year between the old and the new fiscal
/// year end.
const TRANSITION_REPORT_FORMS: [&str; 1] = ["10-KT"];

/// Applies `core/period/q4-from-fy@1` to the series of one entity after
/// `core/dedup/newest-filing@1`, and returns the derived facts.
///
/// For each fiscal year value, the rule looks for the nine-month value of the same concept, unit,
/// and dimensions, whose end lies one quarter before the end of the year. The year must end 47 to
/// 137 days after the nine-month end, so that the fourth quarter rounds to one quarter. If
/// several nine-month values fit, the rule takes the latest end. If both periods have an exact
/// start, the starts must be equal. The fourth quarter is the year less the nine months. It runs
/// from the day after the nine-month end to the year end.
///
/// The rule derives only values in one currency, such as `USD`. Per-share values, share counts,
/// and ratios do not add over periods. The rule derives nothing for a fourth quarter that the
/// series already holds or that a conflict blocks.
///
/// The rule derives nothing from a short year, because its fourth quarter is not the year less
/// nine months. If the year has an exact start, it is short if it spans under 357 days. Otherwise
/// it is short if one of `filings` of the entity is a `10-KT` whose period of report is the year
/// end, or if the year ends under 357 days after the last report under an old fiscal year end.
///
/// The confidence is [`Confidence::Derived`], or the weaker confidence of an input. The fact has
/// `restated` if an input has it. It carries the release of the fiscal year value.
#[must_use]
pub fn derive_q4(series: &Series, filings: &[Filing]) -> Vec<StandardizedFact> {
    let known: BTreeSet<SeriesKey> = series
        .facts
        .iter()
        .map(StandardizedFact::series_key)
        .chain(series.blocked.iter().cloned())
        .collect();
    let short_year_signals = ShortYearSignalsByEntity::of(filings);
    series
        .facts
        .iter()
        .filter(|year| year.unit.is_currency() && year.period.quarters() == YEAR_QUARTERS)
        .filter(|year| !short_year_signals.is_short_year(year))
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

/// The signals of a short year in the filings of each entity.
#[derive(Debug)]
struct ShortYearSignalsByEntity<'a> {
    by_entity: BTreeMap<&'a EntityId, ShortYearSignals>,
}

/// The signals of a short year in the filings of one entity.
#[derive(Debug)]
struct ShortYearSignals {
    /// The periods of report of the transition reports.
    report_ends: BTreeSet<NaiveDate>,
    /// The latest periods of report under an old fiscal year end, one per change of the fiscal
    /// year end.
    old_year_ends: Vec<NaiveDate>,
}

impl<'a> ShortYearSignalsByEntity<'a> {
    fn of(filings: &'a [Filing]) -> Self {
        let mut grouped: BTreeMap<&EntityId, Vec<&Filing>> = BTreeMap::new();
        for filing in filings {
            grouped.entry(&filing.entity).or_default().push(filing);
        }
        let by_entity = grouped
            .into_iter()
            .map(|(entity, filings)| {
                let report_ends = filings
                    .iter()
                    .filter(|filing| is_form(&filing.form, &TRANSITION_REPORT_FORMS))
                    .filter_map(|filing| filing.period_of_report)
                    .collect();
                let signals = ShortYearSignals {
                    report_ends,
                    old_year_ends: changes(&filings)
                        .into_iter()
                        .map(|(after, _)| after)
                        .collect(),
                };
                (entity, signals)
            })
            .collect();
        Self { by_entity }
    }

    /// Returns `true` if `year` is a short year. The day count decides if the exact start is known.
    /// Otherwise a year that ends under 357 days after the last report under an old fiscal year
    /// end is short, because it starts after that report.
    fn is_short_year(&self, year: &StandardizedFact) -> bool {
        let Period::Duration(period) = year.period else {
            return false;
        };
        if let Some(start) = period.start() {
            return is_short_span(start, period.end());
        }
        let end = period.end();
        self.by_entity.get(&year.entity).is_some_and(|signals| {
            signals.report_ends.contains(&end)
                || signals
                    .old_year_ends
                    .iter()
                    .any(|&old_year_end| old_year_end < end && is_short_span(old_year_end, end))
        })
    }
}

/// Returns `true` if the span from `start` to `end` lasts under 357 days.
fn is_short_span(start: NaiveDate, end: NaiveDate) -> bool {
    (end - start).num_days() < SHORT_YEAR_DAYS
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
    use crate::core::elements::CanonicalElement;
    use crate::core::fact::Dimensions;
    use crate::core::unit::Unit;
    use crate::tests::fixtures::sample_fact::{date, filing, standardized};

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

    /// The values of a short year from 1 February to 31 December 2023, without exact starts, as
    /// the Notes data set gives them.
    fn short_year_without_start() -> Series {
        series(vec![
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2023, 12, 31), 4)),
                "330000000",
            ),
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2023, 10, 31), 3)),
                "270000000",
            ),
        ])
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

        let result = summary(derive_q4(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_year_is_a_short_year_of_eleven_months() {
        let facts = series(vec![
            standardized(span(date(2023, 2, 1), date(2023, 12, 31)), "330000000"),
            standardized(span(date(2023, 2, 1), date(2023, 10, 31)), "270000000"),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_a_transition_report_ends_with_the_year() {
        let facts = short_year_without_start();
        let filings = vec![
            filing("10-K", date(2023, 1, 31), "1231"),
            filing("10-KT", date(2023, 12, 31), "1231"),
        ];

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts, &filings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_an_amended_transition_report_ends_with_the_year() {
        let facts = short_year_without_start();
        let filings = vec![filing("10-KT/A", date(2023, 12, 31), "1231")];

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts, &filings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_year_ends_after_a_fiscal_year_end_change() {
        let facts = short_year_without_start();
        let filings = vec![
            filing("10-K", date(2023, 1, 31), "0131"),
            filing("10-K", date(2023, 12, 31), "1231"),
        ];

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts, &filings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_the_year_ends_before_a_fiscal_year_end_change() {
        let facts = series(vec![
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2023, 1, 31), 4)),
                "400000000",
            ),
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2022, 10, 31), 3)),
                "300000000",
            ),
        ]);
        let filings = vec![
            filing("10-K", date(2023, 1, 31), "0131"),
            filing("10-KT", date(2023, 12, 31), "1231"),
        ];

        let expected_result = vec![BigDecimal::from(100_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts, &filings)
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_a_full_year_ends_twelve_months_after_the_last_report_under_the_old_fiscal_year_end()
     {
        let facts = series(vec![
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 6, 30), 4)),
                "400000000",
            ),
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 3, 31), 3)),
                "300000000",
            ),
        ]);
        let filings = vec![
            filing("10-K", date(2022, 12, 31), "1231"),
            filing("10-KT", date(2023, 6, 30), "1231"),
            filing("10-Q", date(2023, 9, 30), "0630"),
        ];

        let expected_result = vec![BigDecimal::from(100_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts, &filings)
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_a_transition_report_of_another_entity_ends_with_the_year()
     {
        let facts = short_year_without_start();
        let mut transition_report = filing("10-KT", date(2023, 12, 31), "1231");
        transition_report.entity = EntityId::new("http://www.sec.gov/CIK", "0000789019")
            .expect("Given a hardcoded scheme and ID, the creation should always succeed");
        let filings = vec![transition_report];

        let expected_result = vec![BigDecimal::from(60_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts, &filings)
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_the_year_has_an_exact_start_and_a_transition_report_ends_with_it()
     {
        let facts = series(vec![
            standardized(span(date(2023, 1, 1), date(2023, 12, 31)), "400000000"),
            standardized(span(date(2023, 1, 1), date(2023, 9, 30)), "300000000"),
        ]);
        let filings = vec![filing("10-KT", date(2023, 12, 31), "1231")];

        let expected_result = vec![BigDecimal::from(100_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts, &filings)
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_the_year_has_53_weeks() {
        let facts = series(vec![
            standardized(span(date(2022, 9, 25), date(2023, 9, 30)), "383285000000"),
            standardized(span(date(2022, 9, 25), date(2023, 7, 1)), "293787000000"),
        ]);

        let expected_result = vec![BigDecimal::from(89_498_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts, &[])
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

        let result: Vec<BigDecimal> = derive_q4(&facts, &[])
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

        let result = derive_q4(&facts, &[]);

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

        let result = summary(derive_q4(&facts, &[]));

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

        let result: Vec<_> = derive_q4(&facts, &[])
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

        let result = derive_q4(&facts, &[]);

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

        let result = derive_q4(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_mark_the_fourth_quarter_restated_when_an_input_is_restated() {
        let year = standardized(fiscal_year(), "391035000000");
        let mut first_nine_months = standardized(nine_months(), "296105000000");
        first_nine_months.restated = true;
        let facts = series(vec![year, first_nine_months]);

        let expected_result = vec![true];

        let result: Vec<bool> = derive_q4(&facts, &[])
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

        let result: Vec<Confidence> = derive_q4(&facts, &[])
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

        let result = derive_q4(&facts, &[]);

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

        let result = derive_q4(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_nine_months_start_in_another_fiscal_year() {
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            standardized(span(date(2023, 9, 24), date(2024, 6, 29)), "296105000000"),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts, &[]);

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

        let result = derive_q4(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_nine_months_are_another_concept() {
        let mut first_nine_months = standardized(nine_months(), "296105000000");
        first_nine_months.concept = CanonicalElement::CostOfRevenue;
        let facts = series(vec![
            standardized(fiscal_year(), "391035000000"),
            first_nine_months,
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts, &[]);

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

        let result = derive_q4(&facts, &[]);

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

        let result = derive_q4(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_nine_months_end_in_the_prior_year() {
        let facts = series(vec![
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 4)),
                "391035000000",
            ),
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2023, 7, 1), 3)),
                "281000000000",
            ),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_latest_nine_month_end_when_two_nine_month_values_fit() {
        let facts = series(vec![
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 4)),
                "391035000000",
            ),
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 6, 1), 3)),
                "250000000000",
            ),
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 6, 29), 3)),
                "296105000000",
            ),
        ]);

        let expected_result = vec![BigDecimal::from(94_930_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts, &[])
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

        let result: Vec<bool> = derive_q4(&facts, &[])
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

        let result: Vec<Confidence> = derive_q4(&facts, &[])
            .into_iter()
            .map(|fact| fact.confidence)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_fourth_quarter_when_the_year_ends_137_days_after_the_nine_months() {
        let facts = series(vec![
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 4)),
                "391035000000",
            ),
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 5, 14), 3)),
                "296105000000",
            ),
        ]);

        let expected_result = vec![BigDecimal::from(94_930_000_000_i64)];

        let result: Vec<BigDecimal> = derive_q4(&facts, &[])
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_year_ends_138_days_after_the_nine_months() {
        let facts = series(vec![
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 4)),
                "391035000000",
            ),
            standardized(
                Period::Duration(Duration::from_end_and_quarters(date(2024, 5, 13), 3)),
                "296105000000",
            ),
        ]);

        let expected_result: Vec<StandardizedFact> = Vec::new();

        let result = derive_q4(&facts, &[]);

        assert_eq!(result, expected_result);
    }
}
