//! # Suspicious Period
//!
//! Provides [`suspicious_periods`], the period check: it flags the standardized values whose period
//! is most likely a typo of the filer, and names another filing that reports the same value for a
//! plausible period.

use std::cmp::Ordering;

use chrono::{Datelike, Days, Months, NaiveDate};

use super::dedup::Series;
use super::filing::{Filing, FilingId};
use super::finding::{Finding, PeriodSignal};
use super::period::{Period, days_of_quarters};
use super::standardized::StandardizedFact;

/// No period starts before 1 January of this year. In the Notes files of 2023 to 2026, every
/// earlier date is a typo or a date the filer tagged by mistake.
const EARLIEST_YEAR: i32 = 1900;

/// The months by which the end of a period can lie away from the period of report of its filing:
/// 100 years. Ends 10 to 100 years ahead are real, such as debt terms or carbon price assumptions.
const MONTHS_FROM_REPORT: u32 = 1200;

/// The years by which the end of a confirming value can lie away from the period of report of the
/// flagged filing. A 10-K compares 3 years, and every confirmation in the measurement ends within
/// 2 years. The limit keeps a common value, such as 0, of another year from confirming by chance.
const CONFIRMATION_YEARS: i32 = 3;

/// Checks the period of each value in `series`, and returns one [`Finding::SuspiciousPeriod`] per
/// value that hits a signal.
///
/// The finding names every signal the value hits. The check reports and never fails. A flagged
/// value keeps its period.
///
/// - [`PeriodSignal::StartBeforeEarliestYear`]: the period starts before 1900. For an instant,
///   the date counts. If the source gives no exact start, the start is the end less the length
///   in quarters of 91.3125 days.
/// - [`PeriodSignal::EndFarFromReport`]: the end lies more than 100 years before or after the
///   period of report of the filing of the value. A filing missing from `filings`, or one with no
///   period of report, gives no such signal.
///
/// The value key of the finding is the first input of the value. Another value confirms the
/// flagged value if it has the same concept, unit, dimensions, and value, hits no signal, ends on
/// the same month and day within 3 years of the period of report of the flagged filing, and comes
/// from another filing. If the flagged filing is missing from `filings` or has no period of
/// report, no value confirms it. A typo in a comparative has its
/// original in an earlier filing, and a typo in a new value has its comparative in a later one,
/// so both count. If several filings confirm, the finding names the newest: the latest `filed`,
/// then the latest `accepted`, then the highest filing ID.
#[must_use]
pub fn suspicious_periods(series: &Series, filings: &[Filing]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for fact in &series.facts {
        let Some(key) = fact.inputs.first() else {
            continue;
        };
        let signals = signals(fact, filings);
        if signals.is_empty() {
            continue;
        }
        findings.push(Finding::SuspiciousPeriod {
            entity: fact.entity.clone(),
            key: key.clone(),
            standard_concept: fact.concept,
            signals,
            confirmed_by: confirmation(fact, &key.filing, series, filings),
        });
    }
    findings
}

/// Returns the signals that the period of `fact` hits, in the order of [`PeriodSignal`].
fn signals(fact: &StandardizedFact, filings: &[Filing]) -> Vec<PeriodSignal> {
    let mut signals = Vec::new();
    if starts_before_earliest_year(&fact.period) {
        signals.push(PeriodSignal::StartBeforeEarliestYear);
    }
    if report_of(fact, filings).is_some_and(|report| is_far_from(fact.period.end(), report)) {
        signals.push(PeriodSignal::EndFarFromReport);
    }
    signals
}

fn starts_before_earliest_year(period: &Period) -> bool {
    let start = match period {
        Period::Instant { end } => Some(*end),
        Period::Duration(duration) => duration.start().or_else(|| {
            let days = u64::try_from(days_of_quarters(duration.quarters())).ok()?;
            duration.end().checked_sub_days(Days::new(days))
        }),
    };
    // A start too far back for a date lies before the earliest year too.
    start.is_none_or(|start| start.year() < EARLIEST_YEAR)
}

fn is_far_from(end: NaiveDate, report: NaiveDate) -> bool {
    let months = Months::new(MONTHS_FROM_REPORT);
    let too_late = report
        .checked_add_months(months)
        .is_some_and(|limit| end > limit);
    let too_early = report
        .checked_sub_months(months)
        .is_some_and(|limit| end < limit);
    too_late || too_early
}

/// Returns the newest other filing that reports the value of `flagged` for a plausible period.
fn confirmation(
    flagged: &StandardizedFact,
    flagged_filing: &FilingId,
    series: &Series,
    filings: &[Filing],
) -> Option<FilingId> {
    let end = flagged.period.end();
    let report = report_of(flagged, filings)?;
    series
        .facts
        .iter()
        .filter(|other| {
            other.entity == flagged.entity
                && other.concept == flagged.concept
                && other.unit == flagged.unit
                && other.dimensions == flagged.dimensions
                && other.value == flagged.value
                && other.period.end().month() == end.month()
                && other.period.end().day() == end.day()
                && (other.period.end().year() - report.year()).abs() <= CONFIRMATION_YEARS
                && signals(other, filings).is_empty()
        })
        .filter_map(|other| other.inputs.first().map(|key| &key.filing))
        .filter(|filing_id| *filing_id != flagged_filing)
        .max_by(|left, right| filing_order(left, right, filings))
        .cloned()
}

/// Returns the period of report of the filing of the first input of `fact`.
fn report_of(fact: &StandardizedFact, filings: &[Filing]) -> Option<NaiveDate> {
    fact.inputs
        .first()
        .and_then(|key| filing(filings, &key.filing))
        .and_then(|filing| filing.period_of_report)
}

fn filing<'a>(filings: &'a [Filing], id: &FilingId) -> Option<&'a Filing> {
    filings.iter().find(|filing| &filing.id == id)
}

/// Orders two filings as `core/dedup/newest-filing@1` does. A filing missing from `filings`
/// ranks below every filing in it.
fn filing_order(left: &FilingId, right: &FilingId, filings: &[Filing]) -> Ordering {
    let time = |id: &FilingId| filing(filings, id).map(|filing| (filing.filed, filing.accepted));
    time(left).cmp(&time(right)).then_with(|| left.cmp(right))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::CanonicalElement;
    use crate::core::period::Duration;
    use crate::tests::fixtures::sample_fact::{date, filing as sample_filing, standardized};

    /// Returns the value `value` of the period that ends on `end` and lasts `quarters`, with no
    /// exact start, reported in `filing`.
    fn value_in(filing: &Filing, end: NaiveDate, quarters: u16, value: &str) -> StandardizedFact {
        let mut fact = standardized(
            Period::Duration(Duration::from_end_and_quarters(end, quarters)),
            value,
        );
        fact.inputs[0].filing = filing.id.clone();
        fact
    }

    /// Returns the instant value `value` on `end`, reported in `filing`.
    fn instant_in(filing: &Filing, end: NaiveDate, value: &str) -> StandardizedFact {
        let mut fact = standardized(Period::Instant { end }, value);
        fact.inputs[0].filing = filing.id.clone();
        fact
    }

    fn series(facts: Vec<StandardizedFact>) -> Series {
        Series {
            facts,
            blocked: Vec::new(),
        }
    }

    fn finding(
        fact: &StandardizedFact,
        signals: &[PeriodSignal],
        confirmed_by: Option<&Filing>,
    ) -> Finding {
        Finding::SuspiciousPeriod {
            entity: fact.entity.clone(),
            key: fact.inputs[0].clone(),
            standard_concept: CanonicalElement::Revenue,
            signals: signals.to_vec(),
            confirmed_by: confirmed_by.map(|filing| filing.id.clone()),
        }
    }

    #[test]
    fn should_flag_start_before_earliest_year_when_duration_starts_before_1900() {
        let report = sample_filing("10-K", date(2025, 12, 31), "1231");
        // 7,297 quarters before the end of 2024 start in the year 200, as in a real 10-K.
        let flagged = value_in(&report, date(2024, 12, 31), 7297, "16218");
        let facts = series(vec![flagged.clone()]);

        let expected_result = vec![finding(
            &flagged,
            &[PeriodSignal::StartBeforeEarliestYear],
            None,
        )];

        let result = suspicious_periods(&facts, &[report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_flag_both_signals_in_one_finding_when_instant_lies_before_1900() {
        let report = sample_filing("10-K", date(2025, 12, 31), "1231");
        let flagged = instant_in(&report, date(205, 12, 31), "21500000");
        let facts = series(vec![flagged.clone()]);

        let expected_result = vec![finding(
            &flagged,
            &[
                PeriodSignal::StartBeforeEarliestYear,
                PeriodSignal::EndFarFromReport,
            ],
            None,
        )];

        let result = suspicious_periods(&facts, &[report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_flag_end_far_from_report_when_end_lies_over_100_years_after_the_report() {
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let flagged = value_in(&report, date(2923, 6, 30), 3603, "50000");
        let facts = series(vec![flagged.clone()]);

        let expected_result = vec![finding(&flagged, &[PeriodSignal::EndFarFromReport], None)];

        let result = suspicious_periods(&facts, &[report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_flag_nothing_when_end_lies_less_than_100_years_after_the_report() {
        let report = sample_filing("10-Q", date(2025, 9, 30), "1231");
        // A debt term of 15 years, as real filings report it.
        let term = value_in(&report, date(2042, 11, 30), 60, "6");
        let facts = series(vec![term]);

        let expected_result: Vec<Finding> = Vec::new();

        let result = suspicious_periods(&facts, &[report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_flag_nothing_when_the_filing_of_the_value_is_unknown() {
        let unknown = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let flagged = value_in(&unknown, date(2923, 6, 30), 3603, "50000");
        let facts = series(vec![flagged]);

        let expected_result: Vec<Finding> = Vec::new();

        let result = suspicious_periods(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_earlier_filing_when_the_original_of_a_comparative_confirms_it() {
        let original = sample_filing("10-Q", date(2023, 6, 30), "1231");
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let quarter = value_in(&original, date(2023, 6, 30), 1, "50000");
        let flagged = value_in(&report, date(2923, 6, 30), 3603, "50000");
        let facts = series(vec![quarter, flagged.clone()]);

        let expected_result = vec![finding(
            &flagged,
            &[PeriodSignal::EndFarFromReport],
            Some(&original),
        )];

        let result = suspicious_periods(&facts, &[original, report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_later_filing_when_a_later_comparative_confirms_the_value() {
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let next = sample_filing("10-Q", date(2025, 6, 30), "1231");
        let flagged = value_in(&report, date(3024, 6, 30), 1, "50000");
        let comparative = value_in(&next, date(2024, 6, 30), 1, "50000");
        let facts = series(vec![comparative, flagged.clone()]);

        let expected_result = vec![finding(
            &flagged,
            &[PeriodSignal::EndFarFromReport],
            Some(&next),
        )];

        let result = suspicious_periods(&facts, &[report, next]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_newest_filing_when_several_filings_confirm_the_value() {
        let original = sample_filing("10-Q", date(2023, 6, 30), "1231");
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let mut amendment = sample_filing("10-Q/A", date(2023, 6, 30), "1231");
        amendment.filed = date(2023, 9, 1);
        // An ID below the ID of the original, so that only the filing date makes it the newest.
        amendment.id = FilingId::new("sec", "0000000000-23-000001")
            .expect("Given a hardcoded source and ID, the creation should always succeed");
        let quarter = value_in(&original, date(2023, 6, 30), 1, "50000");
        let half_year = value_in(&amendment, date(2023, 6, 30), 2, "50000");
        let flagged = value_in(&report, date(2923, 6, 30), 3603, "50000");
        let facts = series(vec![quarter, half_year, flagged.clone()]);

        let expected_result = vec![finding(
            &flagged,
            &[PeriodSignal::EndFarFromReport],
            Some(&amendment),
        )];

        let result = suspicious_periods(&facts, &[original, report, amendment]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_filing_when_no_other_filing_reports_the_value() {
        let original = sample_filing("10-Q", date(2023, 6, 30), "1231");
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let quarter = value_in(&original, date(2023, 6, 30), 1, "49000");
        let flagged = value_in(&report, date(2923, 6, 30), 3603, "50000");
        let facts = series(vec![quarter, flagged.clone()]);

        let expected_result = vec![finding(&flagged, &[PeriodSignal::EndFarFromReport], None)];

        let result = suspicious_periods(&facts, &[original, report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_filing_when_only_the_flagged_filing_reports_the_value() {
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let quarter = value_in(&report, date(2024, 6, 30), 1, "50000");
        let flagged = value_in(&report, date(2924, 6, 30), 3603, "50000");
        let facts = series(vec![quarter, flagged.clone()]);

        let expected_result = vec![finding(&flagged, &[PeriodSignal::EndFarFromReport], None)];

        let result = suspicious_periods(&facts, &[report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_filing_when_the_other_value_ends_on_another_day_of_the_year() {
        let original = sample_filing("10-Q", date(2023, 3, 31), "1231");
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let quarter = value_in(&original, date(2023, 3, 31), 1, "50000");
        let flagged = value_in(&report, date(2923, 6, 30), 3603, "50000");
        let facts = series(vec![quarter, flagged.clone()]);

        let expected_result = vec![finding(&flagged, &[PeriodSignal::EndFarFromReport], None)];

        let result = suspicious_periods(&facts, &[original, report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_filing_when_the_only_other_value_also_hits_a_signal() {
        let first = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let second = sample_filing("10-Q", date(2025, 6, 30), "1231");
        let other_typo = value_in(&first, date(2924, 6, 30), 3603, "50000");
        let flagged = value_in(&second, date(2923, 6, 30), 3599, "50000");
        let facts = series(vec![other_typo.clone(), flagged.clone()]);

        let expected_result = vec![
            finding(&other_typo, &[PeriodSignal::EndFarFromReport], None),
            finding(&flagged, &[PeriodSignal::EndFarFromReport], None),
        ];

        let result = suspicious_periods(&facts, &[first, second]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_flag_nothing_when_end_lies_exactly_100_years_after_the_report() {
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let term = value_in(&report, date(2124, 6, 30), 4, "50000");
        let facts = series(vec![term]);

        let expected_result: Vec<Finding> = Vec::new();

        let result = suspicious_periods(&facts, &[report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_flag_end_far_from_report_when_end_lies_100_years_and_one_day_after_the_report() {
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let flagged = value_in(&report, date(2124, 7, 1), 4, "50000");
        let facts = series(vec![flagged.clone()]);

        let expected_result = vec![finding(&flagged, &[PeriodSignal::EndFarFromReport], None)];

        let result = suspicious_periods(&facts, &[report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_no_filing_when_the_other_value_ends_more_than_3_years_from_the_report() {
        let old = sample_filing("10-K", date(2016, 6, 30), "0630");
        let report = sample_filing("10-Q", date(2024, 6, 30), "1231");
        let year = value_in(&old, date(2015, 6, 30), 4, "0");
        let flagged = value_in(&report, date(2923, 6, 30), 3603, "0");
        let facts = series(vec![year, flagged.clone()]);

        let expected_result = vec![finding(&flagged, &[PeriodSignal::EndFarFromReport], None)];

        let result = suspicious_periods(&facts, &[old, report]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_flag_start_before_earliest_year_when_only_the_exact_start_lies_before_1900() {
        // 131 days round to 1 quarter. The estimate from 1 quarter starts on 1900-01-30, so only
        // the exact start on 1899-12-20 raises the flag.
        let period = Duration::from_dates(date(1899, 12, 20), date(1900, 4, 30))
            .expect("Given an end after the start, the creation should always succeed");
        let flagged = standardized(Period::Duration(period), "50000");
        let facts = series(vec![flagged.clone()]);

        let expected_result = vec![finding(
            &flagged,
            &[PeriodSignal::StartBeforeEarliestYear],
            None,
        )];

        let result = suspicious_periods(&facts, &[]);

        assert_eq!(result, expected_result);
    }
}
