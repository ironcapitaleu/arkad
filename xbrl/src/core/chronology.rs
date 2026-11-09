//! # Chronology
//!
//! Provides [`check_chronology`], the gap check: it orders the standardized values of each entity
//! and concept over time, and reports gaps, overlaps, restatements, and suspicious periods as
//! findings.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use chrono::{Datelike, Months, NaiveDate};

use super::candidate::SeriesKey;
use super::dedup::Series;
use super::elements::CanonicalElement;
use super::fact::Dimensions;
use super::filing::{EntityId, Filing};
use super::finding::{Finding, GapReason};
use super::fiscal_period::FiscalPeriod;
use super::form::Form;
use super::invalid_value::{InvalidValue, InvalidValueReason};
use super::period::days_of_quarters;
use super::suspicious_period::suspicious_periods;
use super::unit::Unit;

/// The days by which two period ends can differ and still name the same period. A 52-53 week
/// fiscal year ends up to 6 days away from the end of its month.
const TOLERANCE_DAYS: i64 = 7;

/// The forms of a quarterly report. A `10-QT` reports a transition period.
const QUARTERLY_FORMS: [&str; 2] = ["10-Q", "10-QT"];

/// The forms of the annual report of a filer that also files quarterly reports.
const DOMESTIC_ANNUAL_FORMS: [&str; 2] = ["10-K", "10-KT"];

/// The forms of an annual report. A `10-KT` reports a transition period.
const ANNUAL_FORMS: [&str; 4] = ["10-K", "10-KT", "20-F", "40-F"];

/// The months between two expected periods of a filer that reports every quarter.
const QUARTER_MONTHS: u32 = 3;

/// The months of a fiscal year.
const YEAR_MONTHS: u32 = 12;

/// The fiscal periods by the number of quarters they end before the fiscal year end.
const FISCAL_PERIODS: [&str; 4] = ["FY", "Q3", "Q2", "Q1"];

/// The standardized values and the findings of the gap check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chronology {
    /// The values after dedup and derivation, unchanged.
    pub series: Series,
    /// The `Gap`, `Overlap`, `Restatement`, and `SuspiciousPeriod` findings, then the findings of
    /// dedup.
    pub findings: Vec<Finding>,
}

/// Checks the values of each entity, concept, unit, and dimensions over time, and returns them
/// with the findings. The check reports and never fails.
///
/// The forms the entity files give the expected periods. An entity that files a `10-Q` expects
/// `Q1`, `Q2`, `Q3`, and `FY` in the years it closes with a `10-K` or `10-KT`. A filer of only
/// `10-K`, `10-KT`, `20-F`, or `40-F` expects `FY`. Another filer expects no period. The period of
/// report of each annual report is a fiscal year end. Between two year ends, the check expects a
/// period every 3 months back from the later end if the annual report for that end is a `10-K` or
/// `10-KT` and the entity filed a quarterly report before that end, otherwise every 12 months.
/// After the last year end, it steps forward up to the latest period of report: every 3 months if
/// the last annual report is a `10-K` or `10-KT` and the entity filed any quarterly report,
/// otherwise every 12 months. The period of report of a quarterly report replaces the expected end
/// within 7 days of it, or adds one. Every timeline of the entity expects the same periods.
///
/// - A **gap** is an expected period between the first and the last filled period, with no value
///   that ends within 7 days of it. A value of any length fills it. A blocked value gives the
///   reason [`GapReason::Conflict`]. Otherwise a gap in the 12 months after the last report
///   period under an old fiscal year end gives the reason [`GapReason::FiscalYearChange`].
/// - An **overlap** is a pair of values of the same length whose ends lie closer than that length,
///   less 7 days.
/// - A **restatement** is a value with `restated`.
/// - A **suspicious period** is a period that hits a signal of [`suspicious_periods`].
///
/// `findings` holds the findings of dedup, for example [`Finding::Conflict`], after the findings
/// of the check.
#[must_use]
pub fn check_chronology(series: Series, filings: &[Filing], findings: Vec<Finding>) -> Chronology {
    let mut checked = Vec::new();
    let mut calendars: BTreeMap<EntityId, Calendar> = BTreeMap::new();
    for (timeline, ends) in timelines(&series) {
        let calendar = calendars
            .entry(timeline.entity.clone())
            .or_insert_with(|| Calendar::of(&timeline.entity, filings));
        checked.extend(gaps(&timeline, &ends, calendar));
        checked.extend(overlaps(&timeline, &ends));
    }
    checked.extend(
        series
            .facts
            .iter()
            .filter(|fact| fact.restated)
            .map(|fact| Finding::Restatement {
                key: fact.series_key(),
            }),
    );
    checked.extend(suspicious_periods(&series, filings));
    checked.extend(findings);
    Chronology {
        series,
        findings: checked,
    }
}

/// The values of one entity, concept, unit, and dimensions over time.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Timeline {
    entity: EntityId,
    concept: CanonicalElement,
    unit: Unit,
    dimensions: Dimensions,
}

impl Timeline {
    fn of(key: &SeriesKey) -> Self {
        Self {
            entity: key.entity.clone(),
            concept: key.concept,
            unit: key.unit.clone(),
            dimensions: key.dimensions.clone(),
        }
    }

    fn key(&self, period_end: NaiveDate, quarters: u16) -> SeriesKey {
        SeriesKey {
            entity: self.entity.clone(),
            concept: self.concept,
            period_end,
            quarters,
            unit: self.unit.clone(),
            dimensions: self.dimensions.clone(),
        }
    }
}

/// The period ends of one timeline: of the values, with their length in quarters, and of the
/// blocked keys.
#[derive(Debug, Default)]
struct Ends {
    reported: BTreeSet<(NaiveDate, u16)>,
    blocked: BTreeSet<NaiveDate>,
}

fn timelines(series: &Series) -> BTreeMap<Timeline, Ends> {
    let mut timelines: BTreeMap<Timeline, Ends> = BTreeMap::new();
    for fact in &series.facts {
        let key = fact.series_key();
        let ends = timelines.entry(Timeline::of(&key)).or_default();
        ends.reported.insert((key.period_end, key.quarters));
    }
    for key in &series.blocked {
        let ends = timelines.entry(Timeline::of(key)).or_default();
        ends.blocked.insert(key.period_end);
    }
    timelines
}

fn gaps(timeline: &Timeline, ends: &Ends, calendar: &Calendar) -> Vec<Finding> {
    let filled = |expected: &Expected| {
        ends.reported
            .iter()
            .any(|(end, _)| is_near(*end, expected.end))
    };
    let (Some(first), Some(last)) = (
        calendar.expected.iter().position(filled),
        calendar.expected.iter().rposition(filled),
    ) else {
        return Vec::new();
    };
    calendar.expected[first..last]
        .iter()
        .filter(|expected| !filled(expected))
        .map(|expected| Finding::Gap {
            entity: timeline.entity.clone(),
            concept: timeline.concept,
            unit: timeline.unit.clone(),
            dimensions: timeline.dimensions.clone(),
            period_end: expected.end,
            fiscal_period: expected.fiscal_period.clone(),
            reason: calendar.reason(expected.end, &ends.blocked),
        })
        .collect()
}

fn overlaps(timeline: &Timeline, ends: &Ends) -> Vec<Finding> {
    let mut by_length: BTreeMap<u16, Vec<NaiveDate>> = BTreeMap::new();
    for &(end, quarters) in &ends.reported {
        by_length.entry(quarters).or_default().push(end);
    }
    let mut findings = Vec::new();
    for (quarters, ends) in by_length {
        let days = days_of_quarters(quarters);
        for (index, &first) in ends.iter().enumerate() {
            for &second in ends[index + 1..]
                .iter()
                .take_while(|second| (**second - first).num_days() < days - TOLERANCE_DAYS)
            {
                findings.push(Finding::Overlap {
                    first: timeline.key(first, quarters),
                    second: timeline.key(second, quarters),
                });
            }
        }
    }
    findings
}

fn is_near(left: NaiveDate, right: NaiveDate) -> bool {
    (left - right).num_days().abs() <= TOLERANCE_DAYS
}

/// One expected period: its end and its fiscal period, if it is known.
#[derive(Debug, Clone)]
struct Expected {
    end: NaiveDate,
    fiscal_period: Option<FiscalPeriod>,
}

/// The expected periods of one entity, sorted by end, and the spans in which it changed its
/// fiscal year end.
#[derive(Debug, Default)]
struct Calendar {
    expected: Vec<Expected>,
    changes: Vec<(NaiveDate, NaiveDate)>,
}

impl Calendar {
    fn of(entity: &EntityId, filings: &[Filing]) -> Self {
        let filings: Vec<&Filing> = filings
            .iter()
            .filter(|filing| &filing.entity == entity)
            .collect();
        let quarterly: Vec<&Filing> = filings
            .iter()
            .copied()
            .filter(|filing| is_form(&filing.form, &QUARTERLY_FORMS))
            .collect();
        let year_ends: BTreeSet<NaiveDate> = filings
            .iter()
            .filter(|filing| is_form(&filing.form, &ANNUAL_FORMS))
            .filter_map(|filing| filing.period_of_report)
            .collect();
        let mut expected = Vec::new();
        let mut previous: Option<NaiveDate> = None;
        for &year_end in &year_ends {
            if let Some(previous) = previous {
                let step = step_before(&filings, &quarterly, year_end);
                expected.extend(steps_back(previous, year_end, step));
            }
            expected.push(Expected {
                end: year_end,
                fiscal_period: fiscal_period(0),
            });
            previous = Some(year_end);
        }
        let latest = filings
            .iter()
            .filter_map(|filing| filing.period_of_report)
            .max();
        if let (Some(last), Some(latest)) = (previous, latest) {
            let step = step(is_domestic_at(&filings, last), !quarterly.is_empty());
            expected.extend(steps_forward(last, latest, step));
        }
        for filing in &quarterly {
            let Some(report) = filing.period_of_report else {
                continue;
            };
            match expected.iter_mut().find(|known| is_near(known.end, report)) {
                Some(known) => {
                    known.end = report;
                    if filing.fiscal_period.is_some() {
                        known.fiscal_period.clone_from(&filing.fiscal_period);
                    }
                }
                None => expected.push(Expected {
                    end: report,
                    fiscal_period: filing.fiscal_period.clone(),
                }),
            }
        }
        expected.sort_by_key(|known| known.end);
        Self {
            expected,
            changes: changes(&filings),
        }
    }

    /// Returns the reason of a gap at `end`. [`GapReason::Conflict`] wins over
    /// [`GapReason::FiscalYearChange`].
    fn reason(&self, end: NaiveDate, blocked: &BTreeSet<NaiveDate>) -> Option<GapReason> {
        if blocked.iter().any(|blocked| is_near(*blocked, end)) {
            return Some(GapReason::Conflict);
        }
        self.changes
            .iter()
            .any(|(after, until)| *after < end && end <= *until)
            .then_some(GapReason::FiscalYearChange)
    }
}

/// Returns `true` if the form, without the amendment suffix, is one of `forms`.
pub(super) fn is_form(form: &Form, forms: &[&str]) -> bool {
    let base = form
        .as_str()
        .strip_suffix(Form::AMENDMENT_SUFFIX)
        .unwrap_or(form.as_str());
    forms.contains(&base)
}

/// Returns the step in months for the span that ends at the fiscal year end `year_end`: 3 if the
/// annual report for that end is a `10-K` or `10-KT` and the entity filed a quarterly report
/// before that end, otherwise 12.
fn step_before(filings: &[&Filing], quarterly: &[&Filing], year_end: NaiveDate) -> u32 {
    let is_quarterly = quarterly
        .iter()
        .filter_map(|filing| filing.period_of_report)
        .any(|report| report < year_end);
    step(is_domestic_at(filings, year_end), is_quarterly)
}

/// Returns the step in months: 3 if the year closes with a `10-K` or `10-KT` and the entity files
/// quarterly reports, otherwise 12.
const fn step(is_domestic: bool, is_quarterly: bool) -> u32 {
    if is_domestic && is_quarterly {
        QUARTER_MONTHS
    } else {
        YEAR_MONTHS
    }
}

/// Returns `true` if the annual report for the fiscal year end `year_end` is a `10-K` or `10-KT`.
fn is_domestic_at(filings: &[&Filing], year_end: NaiveDate) -> bool {
    filings.iter().any(|filing| {
        filing.period_of_report == Some(year_end) && is_form(&filing.form, &DOMESTIC_ANNUAL_FORMS)
    })
}

/// Returns the expected periods before `year_end`, every `step` months back, that lie more than
/// 7 days after `previous`.
fn steps_back(previous: NaiveDate, year_end: NaiveDate, step: u32) -> Vec<Expected> {
    (1..)
        .map(|count| step * count)
        .map_while(|months| {
            let end = year_end.checked_sub_months(Months::new(months))?;
            ((end - previous).num_days() > TOLERANCE_DAYS).then(|| Expected {
                end,
                fiscal_period: fiscal_period(months),
            })
        })
        .collect()
}

/// Returns the expected periods after the last fiscal year end `last`, every `step` months, up
/// to 7 days after the latest period of report.
fn steps_forward(last: NaiveDate, latest: NaiveDate, step: u32) -> Vec<Expected> {
    (1..)
        .map(|count| step * count)
        .map_while(|months| {
            let end = last.checked_add_months(Months::new(months))?;
            ((end - latest).num_days() <= TOLERANCE_DAYS).then(|| Expected {
                end,
                fiscal_period: fiscal_period((YEAR_MONTHS - months % YEAR_MONTHS) % YEAR_MONTHS),
            })
        })
        .collect()
}

/// Returns the fiscal period that ends `months` before the fiscal year end.
fn fiscal_period(months: u32) -> Option<FiscalPeriod> {
    let index = usize::try_from(months % YEAR_MONTHS / QUARTER_MONTHS).ok()?;
    FiscalPeriod::new(*FISCAL_PERIODS.get(index)?).ok()
}

/// Returns the spans in which the entity changed its fiscal year end. A span runs from the latest
/// period of report under the old year end, exclusive, for 12 months.
///
/// The filings count in the order they were filed, because a filing states the fiscal year end as
/// of its filing date. An amendment does not count, because it can state the end of the year it
/// amends.
pub(super) fn changes(filings: &[&Filing]) -> Vec<(NaiveDate, NaiveDate)> {
    let mut filings: Vec<&Filing> = filings
        .iter()
        .copied()
        .filter(|filing| !filing.is_amendment)
        .collect();
    filings.sort_by(|left, right| {
        (left.filed, left.accepted, &left.id).cmp(&(right.filed, right.accepted, &right.id))
    });
    let mut changes = Vec::new();
    let mut current: Option<(FiscalYearEnd, Option<NaiveDate>)> = None;
    for filing in filings {
        let Some(year_end) = filing
            .company
            .fiscal_year_end
            .as_deref()
            .and_then(|text| FiscalYearEnd::from_str(text).ok())
        else {
            continue;
        };
        let report = filing.period_of_report;
        current = Some(match current {
            Some((old, latest)) if old.differs_from(year_end) => {
                if let Some(after) = latest {
                    changes.extend(
                        after
                            .checked_add_months(Months::new(YEAR_MONTHS))
                            .map(|until| (after, until)),
                    );
                }
                (year_end, report)
            }
            Some((_, latest)) => (year_end, latest.max(report)),
            None => (year_end, report),
        });
    }
    changes
}

/// A fiscal year end: a month and a day, as the day of a leap year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FiscalYearEnd(u32);

impl FiscalYearEnd {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "FiscalYearEnd";

    /// A leap year, so that `0229` is a valid fiscal year end.
    const LEAP_YEAR: &'static str = "2000";

    /// The forms of the text after [`Self::LEAP_YEAR`]: `0930` as in the Notes data set, and
    /// `--09-30` as in an instance document.
    const FORMATS: [&'static str; 2] = ["%Y%m%d", "%Y--%m-%d"];

    /// The form that [`InvalidValueReason::NotADate`] names.
    const EXPECTED_FORM: &'static str = "%m%d or --%m-%d";

    /// The days of a leap year.
    const LEAP_YEAR_DAYS: u32 = 366;

    /// Returns `true` if the two year ends lie more than 7 days apart, around the year.
    fn differs_from(self, other: Self) -> bool {
        let distance = self.0.abs_diff(other.0);
        i64::from(distance.min(Self::LEAP_YEAR_DAYS - distance)) > TOLERANCE_DAYS
    }
}

impl FromStr for FiscalYearEnd {
    type Err = InvalidValue;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let dated = format!("{}{text}", Self::LEAP_YEAR);
        Self::FORMATS
            .iter()
            .find_map(|format| NaiveDate::parse_from_str(&dated, format).ok())
            .map(|date| Self(date.ordinal()))
            .ok_or_else(|| {
                InvalidValue::new(
                    Self::TYPE_NAME,
                    InvalidValueReason::NotADate(Self::EXPECTED_FORM),
                    text,
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::finding::PeriodSignal;
    use crate::core::period::{Duration, Period};
    use crate::core::standardized::StandardizedFact;
    use crate::tests::fixtures::sample_fact::{date, fact, filing, source, standardized};

    fn value(end: NaiveDate, quarters: u16) -> StandardizedFact {
        standardized(
            Period::Duration(Duration::from_end_and_quarters(end, quarters)),
            "100",
        )
    }

    fn gap(end: NaiveDate, fiscal_period: &str, reason: Option<GapReason>) -> Finding {
        let template = value(end, 1);
        Finding::Gap {
            entity: template.entity,
            concept: template.concept,
            unit: template.unit,
            dimensions: Dimensions::default(),
            period_end: end,
            fiscal_period: Some(FiscalPeriod::new(fiscal_period).expect(
                "Given a hardcoded period without whitespace, the creation should always succeed",
            )),
            reason,
        }
    }

    fn series(facts: Vec<StandardizedFact>, blocked: Vec<SeriesKey>) -> Series {
        Series { facts, blocked }
    }

    /// The filings of a 10-Q filer with the fiscal year end `0930`, from the year 2023 to the
    /// year 2024.
    fn quarterly_filings() -> Vec<Filing> {
        vec![
            filing("10-K", date(2023, 9, 30), "0930"),
            filing("10-Q", date(2023, 12, 30), "0930"),
            filing("10-Q", date(2024, 3, 30), "0930"),
            filing("10-Q", date(2024, 6, 29), "0930"),
            filing("10-K", date(2024, 9, 28), "0930"),
        ]
    }

    /// The filings of a filer that moves its fiscal year end from `1231` to `0630`.
    fn fiscal_year_change_filings() -> Vec<Filing> {
        vec![
            filing("10-K", date(2022, 12, 31), "1231"),
            filing("10-Q", date(2023, 3, 31), "1231"),
            filing("10-KT", date(2023, 6, 30), "0630"),
            filing("10-Q", date(2023, 9, 30), "0630"),
        ]
    }

    fn conflict() -> Finding {
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        Finding::Conflict {
            entity: value(date(2024, 3, 30), 1).entity,
            key: fact(&filing, "Revenues", "1").key(),
            ranks: vec![1],
            standard_concept: Some(CanonicalElement::Revenue),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_chronology() {
        implements_auto_traits::<Chronology>();
    }

    #[test]
    fn should_report_gap_when_quarterly_filer_has_no_q2_value() {
        let facts = vec![
            value(date(2023, 9, 30), 4),
            value(date(2023, 12, 30), 1),
            value(date(2024, 6, 29), 1),
            value(date(2024, 9, 28), 4),
        ];
        let filings = quarterly_filings();

        let expected_result = vec![gap(date(2024, 3, 30), "Q2", None)];

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_gap_when_series_starts_after_the_first_expected_period() {
        let facts = vec![value(date(2024, 6, 29), 1), value(date(2024, 9, 28), 4)];
        let filings = quarterly_filings();

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_gap_when_series_ends_before_the_last_expected_period() {
        let facts = vec![value(date(2023, 9, 30), 4), value(date(2023, 12, 30), 1)];
        let filings = quarterly_filings();

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expect_a_fiscal_year_when_annual_filer_skips_a_year() {
        let facts = vec![value(date(2022, 9, 24), 4), value(date(2024, 9, 28), 4)];
        let filings = vec![
            filing("10-K", date(2022, 9, 24), "0930"),
            filing("10-K", date(2024, 9, 28), "0930"),
        ];

        let expected_result = vec![gap(date(2023, 9, 28), "FY", None)];

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_reason_fiscal_year_change_when_gap_follows_a_new_fiscal_year_end() {
        let facts = vec![
            value(date(2022, 12, 31), 4),
            value(date(2023, 3, 31), 1),
            value(date(2023, 9, 30), 1),
        ];
        let filings = fiscal_year_change_filings();

        let expected_result = vec![gap(
            date(2023, 6, 30),
            "FY",
            Some(GapReason::FiscalYearChange),
        )];

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_reason_conflict_when_blocked_period_lies_between_values() {
        let facts = vec![value(date(2023, 9, 30), 4), value(date(2024, 6, 29), 1)];
        let blocked = vec![value(date(2024, 3, 30), 1).series_key()];
        let filings = quarterly_filings();

        let expected_result = vec![
            gap(date(2023, 12, 30), "Q1", None),
            gap(date(2024, 3, 30), "Q2", Some(GapReason::Conflict)),
        ];

        let result = check_chronology(series(facts, blocked), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_only_the_conflict_when_blocked_period_lies_at_the_end() {
        let facts = vec![value(date(2023, 9, 30), 4), value(date(2023, 12, 30), 1)];
        let blocked = vec![value(date(2024, 3, 30), 1).series_key()];
        let filings = quarterly_filings();
        let findings = vec![conflict()];

        let expected_result = vec![conflict()];

        let result = check_chronology(series(facts, blocked), &filings, findings).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_reason_conflict_when_blocked_period_lies_in_fiscal_year_change() {
        let facts = vec![
            value(date(2022, 12, 31), 4),
            value(date(2023, 3, 31), 1),
            value(date(2023, 9, 30), 1),
        ];
        let blocked = vec![value(date(2023, 6, 30), 1).series_key()];
        let filings = fiscal_year_change_filings();

        let expected_result = vec![gap(date(2023, 6, 30), "FY", Some(GapReason::Conflict))];

        let result = check_chronology(series(facts, blocked), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_overlap_when_two_fiscal_years_overlap() {
        let first = value(date(2022, 12, 31), 4);
        let second = value(date(2023, 6, 30), 4);

        let expected_result = vec![Finding::Overlap {
            first: first.series_key(),
            second: second.series_key(),
        }];

        let result =
            check_chronology(series(vec![first, second], Vec::new()), &[], Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_overlap_when_52_53_week_years_follow_each_other() {
        let facts = vec![value(date(2023, 9, 30), 4), value(date(2024, 9, 28), 4)];

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_chronology(series(facts, Vec::new()), &[], Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_restatement_when_value_is_restated() {
        let mut restated = value(date(2024, 9, 28), 4);
        restated.restated = true;

        let expected_result = vec![Finding::Restatement {
            key: restated.series_key(),
        }];

        let result = check_chronology(series(vec![restated], Vec::new()), &[], Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_instance_form_to_day_of_leap_year_when_end_is_september_30() {
        let instance = "--09-30";

        let expected_result = Ok(FiscalYearEnd(274));

        let result = FiscalYearEnd::from_str(instance);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_fiscal_year_end_when_text_is_not_a_date() {
        let text = "Sept";

        let expected_result = Err(InvalidValue::new(
            "FiscalYearEnd",
            InvalidValueReason::NotADate("%m%d or --%m-%d"),
            "Sept",
        ));

        let result = FiscalYearEnd::from_str(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_change_fiscal_year_end_when_52_53_week_year_drifts() {
        let old = FiscalYearEnd::from_str("--09-24")
            .expect("Given a hardcoded month and day, the parse should always succeed");
        let new = FiscalYearEnd::from_str("--09-30")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = false;

        let result = old.differs_from(new);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_notes_form_to_day_of_leap_year_when_end_is_september_30() {
        let notes = "0930";

        let expected_result = Ok(FiscalYearEnd(274));

        let result = FiscalYearEnd::from_str(notes);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_change_fiscal_year_end_when_month_changes() {
        let old = FiscalYearEnd::from_str("1231")
            .expect("Given a hardcoded month and day, the parse should always succeed");
        let new = FiscalYearEnd::from_str("0630")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = true;

        let result = old.differs_from(new);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_change_fiscal_year_end_when_end_drifts_across_new_year() {
        let old = FiscalYearEnd::from_str("1231")
            .expect("Given a hardcoded month and day, the parse should always succeed");
        let new = FiscalYearEnd::from_str("0102")
            .expect("Given a hardcoded month and day, the parse should always succeed");

        let expected_result = false;

        let result = old.differs_from(new);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_amendment_as_its_form_when_form_has_suffix() {
        let form = Form::new("10-K/A")
            .expect("Given a hardcoded form type, the creation should always succeed");

        let expected_result = true;

        let result = is_form(&form, &ANNUAL_FORMS);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expect_no_quarters_when_filer_files_no_quarterly_report_in_the_year() {
        let facts = vec![
            value(date(2021, 12, 31), 4),
            value(date(2022, 12, 31), 4),
            value(date(2023, 3, 31), 1),
        ];
        let filings = vec![
            filing("20-F", date(2021, 12, 31), "1231"),
            filing("10-K", date(2022, 12, 31), "1231"),
            filing("10-Q", date(2023, 3, 31), "1231"),
        ];

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_gap_when_quarterly_report_after_the_last_year_end_never_came() {
        let facts = vec![value(date(2024, 9, 28), 4), value(date(2025, 3, 29), 1)];
        let mut filings = quarterly_filings();
        filings.push(filing("10-Q", date(2025, 3, 29), "0930"));

        let expected_result = vec![gap(date(2024, 12, 28), "Q1", None)];

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_label_gap_by_the_filing_when_quarterly_report_names_its_period() {
        let facts = vec![value(date(2022, 12, 31), 4), value(date(2023, 6, 30), 2)];
        let mut report = filing("10-Q", date(2023, 3, 31), "1231");
        report.fiscal_period = Some(FiscalPeriod::new("Q1").expect(
            "Given a hardcoded period without whitespace, the creation should always succeed",
        ));
        let filings = vec![
            filing("10-K", date(2022, 12, 31), "1231"),
            report,
            filing("10-KT", date(2023, 6, 30), "0630"),
            filing("10-Q", date(2023, 9, 30), "0630"),
        ];

        let expected_result = vec![gap(date(2023, 3, 31), "Q1", None)];

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_ignore_amendment_when_it_states_the_old_fiscal_year_end() {
        let mut amendment = filing("10-K/A", date(2022, 12, 31), "1231");
        amendment.filed = date(2023, 11, 15);
        let mut filings = fiscal_year_change_filings();
        filings.push(amendment);
        let filings: Vec<&Filing> = filings.iter().collect();

        let expected_result = vec![(date(2023, 3, 31), date(2024, 3, 31))];

        let result = changes(&filings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_overlap_when_values_are_instants() {
        let facts = vec![
            standardized(
                Period::Instant {
                    end: date(2024, 9, 28),
                },
                "1",
            ),
            standardized(
                Period::Instant {
                    end: date(2024, 9, 30),
                },
                "1",
            ),
        ];

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_chronology(series(facts, Vec::new()), &[], Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_overlaps_then_restatements_then_dedup_findings() {
        let first = value(date(2022, 12, 31), 4);
        let mut second = value(date(2023, 6, 30), 4);
        second.restated = true;
        let findings = vec![conflict()];

        let expected_result = vec![
            Finding::Overlap {
                first: first.series_key(),
                second: second.series_key(),
            },
            Finding::Restatement {
                key: second.series_key(),
            },
            conflict(),
        ];

        let result =
            check_chronology(series(vec![first, second], Vec::new()), &[], findings).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_restatements_then_suspicious_periods_then_dedup_findings() {
        let mut restated = value(date(2023, 6, 30), 4);
        restated.restated = true;
        // 7,297 quarters start in the year 200.
        let suspicious = value(date(2024, 12, 31), 7297);
        let findings = vec![conflict()];

        let expected_result = vec![
            Finding::Restatement {
                key: restated.series_key(),
            },
            Finding::SuspiciousPeriod {
                entity: suspicious.entity.clone(),
                key: suspicious.inputs[0].clone(),
                standard_concept: CanonicalElement::Revenue,
                signals: vec![PeriodSignal::StartBeforeEarliestYear],
                confirmed_by: None,
            },
            conflict(),
        ];

        let result = check_chronology(
            series(vec![restated, suspicious], Vec::new()),
            &[],
            findings,
        )
        .findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_gaps_when_quarterly_filer_files_no_quarterly_report_for_a_year() {
        let facts = vec![
            value(date(2022, 12, 31), 4),
            value(date(2023, 12, 31), 4),
            value(date(2024, 3, 31), 1),
        ];
        let filings = vec![
            filing("10-Q", date(2022, 3, 31), "1231"),
            filing("10-K", date(2022, 12, 31), "1231"),
            filing("10-K", date(2023, 12, 31), "1231"),
            filing("10-Q", date(2024, 3, 31), "1231"),
        ];

        let expected_result = vec![
            gap(date(2023, 3, 31), "Q1", None),
            gap(date(2023, 6, 30), "Q2", None),
            gap(date(2023, 9, 30), "Q3", None),
        ];

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expect_no_quarters_when_filer_moves_from_10_k_to_20_f() {
        let facts = vec![
            value(date(2022, 12, 31), 4),
            value(date(2023, 12, 31), 4),
            value(date(2024, 12, 31), 4),
        ];
        let filings = vec![
            filing("10-K", date(2021, 12, 31), "1231"),
            filing("10-Q", date(2022, 3, 31), "1231"),
            filing("10-K", date(2022, 12, 31), "1231"),
            filing("20-F", date(2023, 12, 31), "1231"),
            filing("20-F", date(2024, 12, 31), "1231"),
        ];

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expect_no_quarters_after_the_last_year_end_when_last_annual_report_is_a_20_f() {
        let facts = vec![value(date(2023, 12, 31), 4), value(date(2024, 6, 30), 2)];
        let filings = vec![
            filing("10-K", date(2021, 12, 31), "1231"),
            filing("10-Q", date(2022, 3, 31), "1231"),
            filing("10-K", date(2022, 12, 31), "1231"),
            filing("20-F", date(2023, 12, 31), "1231"),
            filing("6-K", date(2024, 6, 30), "1231"),
        ];

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_chronology(series(facts, Vec::new()), &filings, Vec::new()).findings;

        assert_eq!(result, expected_result);
    }
}
