//! # Dedup
//!
//! Provides the two dedup rules of the standardized layer: [`more_precise`] picks one copy of a
//! value key in one filing, and [`newest_filing`] picks one candidate across filings.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use bigdecimal::{BigDecimal, RoundingMode};
use chrono::{DateTime, Utc};

use super::candidate::{Candidate, CandidateValue, SeriesKey};
use super::fact::{AsIsFact, Decimals, FactValue, FilingSource};
use super::standardized::StandardizedFact;

/// The ID of the rule that picks one copy of a value key in one filing.
pub const MORE_PRECISE: &str = "core/dedup/more-precise@1";

/// The ID of the rule that picks one candidate of a standard concept across filings.
pub const NEWEST_FILING: &str = "core/dedup/newest-filing@1";

/// The as-is facts of one filing after [`more_precise`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Copies<'a> {
    /// The lowest rank of each value key whose ranks agree with it.
    pub facts: Vec<&'a AsIsFact>,
    /// The ranks of each value key with a rank that disagrees with the lowest rank, sorted by rank.
    pub conflicts: Vec<Vec<&'a AsIsFact>>,
}

/// Applies `core/dedup/more-precise@1` to the as-is facts of one filing.
///
/// The rule groups the facts by value key without `copy_rank`, and compares every rank of a group
/// with the lowest rank, which is rank 0. Only the pairs with rank 0 count, so the result does not
/// depend on the order of the checks. If every rank agrees, the group keeps rank 0. Otherwise the
/// group keeps every rank as a conflict.
///
/// Two numeric values agree if they are equal after rounding both to the precision of the less
/// precise one. If either has no `decimals`, they agree only if they are equal. Two values that
/// are not both numeric agree only if they are equal.
#[must_use]
pub fn more_precise(facts: &[AsIsFact]) -> Copies<'_> {
    let mut groups: BTreeMap<_, Vec<&AsIsFact>> = BTreeMap::new();
    for fact in facts {
        let mut key = fact.key();
        key.copy_rank = 0;
        groups.entry(key).or_default().push(fact);
    }
    let mut copies = Copies {
        facts: Vec::new(),
        conflicts: Vec::new(),
    };
    for mut ranks in groups.into_values() {
        ranks.sort_by_key(|fact| fact.copy_rank);
        let first = ranks[0];
        if ranks[1..].iter().all(|copy| facts_agree(first, copy)) {
            copies.facts.push(first);
        } else {
            copies.conflicts.push(ranks);
        }
    }
    copies
}

/// The standardized facts of one entity after [`newest_filing`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Series {
    /// One fact per series key whose newest candidate reports a value, sorted by series key.
    pub facts: Vec<StandardizedFact>,
    /// The series keys whose newest candidate is blocked, sorted. Tiers 3 and 4 derive no value for
    /// them.
    pub blocked: Vec<SeriesKey>,
}

/// Applies `core/dedup/newest-filing@1` to the candidates of one entity.
///
/// The rule groups the candidates by [`SeriesKey`] and takes the candidate of the newest filing:
/// the latest `filed`, then the latest `accepted`, then the highest filing ID. A filing with an
/// `accepted` time ranks above a filing of the same day without one, so the order is total.
///
/// If the newest candidate is blocked, the series key gets no fact and goes to [`Series::blocked`]. The rule does not fall back to
/// an older filing, because the newer filing can hold the restatement. A fact has `restated` if an
/// older filing reports a value that disagrees with it.
#[must_use]
pub fn newest_filing(candidates: &[Candidate]) -> Series {
    let mut series: BTreeMap<SeriesKey, Vec<&Candidate>> = BTreeMap::new();
    for candidate in candidates {
        series
            .entry(candidate.series_key())
            .or_default()
            .push(candidate);
    }
    let mut result = Series {
        facts: Vec::new(),
        blocked: Vec::new(),
    };
    for (key, group) in series {
        match standardize(&group) {
            Some(fact) => result.facts.push(fact),
            None => result.blocked.push(key),
        }
    }
    result
}

/// Applies `core/dedup/newest-filing@1` to the candidates of the filings known at `as_of`.
///
/// A filing is known if it was accepted at or before `as_of`. A filing without an `accepted` time
/// is known if it was filed on or before the date of `as_of`.
#[must_use]
pub fn newest_filing_as_of(candidates: &[Candidate], as_of: DateTime<Utc>) -> Series {
    let known: Vec<Candidate> = candidates
        .iter()
        .filter(|candidate| is_known_at(&candidate.source, as_of))
        .cloned()
        .collect();
    newest_filing(&known)
}

/// Returns the fact of the newest candidate, or `None` if that candidate is blocked.
fn standardize(group: &[&Candidate]) -> Option<StandardizedFact> {
    let newest = group
        .iter()
        .copied()
        .max_by(|left, right| filing_order(&left.source, &right.source))?;
    let CandidateValue::Reported {
        value,
        decimals,
        confidence,
    } = &newest.value
    else {
        return None;
    };
    let restated = group.iter().any(|older| match &older.value {
        CandidateValue::Reported {
            value: older_value,
            decimals: older_decimals,
            ..
        } => !numbers_agree(value, *decimals, older_value, *older_decimals),
        CandidateValue::Blocked => false,
    });
    Some(StandardizedFact {
        entity: newest.entity.clone(),
        concept: newest.concept,
        period: newest.period,
        unit: newest.unit.clone(),
        dimensions: newest.dimensions.clone(),
        value: value.clone(),
        confidence: *confidence,
        rule: newest.rule.clone(),
        release: newest.release.clone(),
        inputs: newest.inputs.clone(),
        restated,
    })
}

fn filing_order(left: &FilingSource, right: &FilingSource) -> Ordering {
    left.filed
        .cmp(&right.filed)
        .then_with(|| left.accepted.cmp(&right.accepted))
        .then_with(|| left.filing.cmp(&right.filing))
}

fn is_known_at(source: &FilingSource, as_of: DateTime<Utc>) -> bool {
    source.accepted.map_or_else(
        || source.filed <= as_of.date_naive(),
        |accepted| accepted <= as_of,
    )
}

/// Returns `true` if the value of `copy` agrees with the value of `first`, as
/// [`more_precise`] defines it.
pub(super) fn facts_agree(first: &AsIsFact, copy: &AsIsFact) -> bool {
    match (&first.value, &copy.value) {
        (FactValue::Numeric(left), FactValue::Numeric(right)) => {
            numbers_agree(left, first.decimals, right, copy.decimals)
        }
        (left, right) => left == right,
    }
}

fn numbers_agree(
    left: &BigDecimal,
    left_decimals: Option<Decimals>,
    right: &BigDecimal,
    right_decimals: Option<Decimals>,
) -> bool {
    match (left_decimals, right_decimals) {
        (Some(left_decimals), Some(right_decimals)) => match left_decimals.min(right_decimals) {
            Decimals::Finite(places) => {
                let places = i64::from(places);
                left.with_scale_round(places, RoundingMode::HalfEven)
                    == right.with_scale_round(places, RoundingMode::HalfEven)
            }
            Decimals::Infinite => left == right,
        },
        _ => left == right,
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::confidence::Confidence;
    use crate::core::period::{Duration, Period};
    use crate::tests::fixtures::sample_fact::{candidate, date, fact, source};

    fn moment(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, 0, 0)
            .single()
            .expect("Given a hardcoded valid time in UTC, the creation should always succeed")
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_copies() {
        implements_auto_traits::<Copies<'static>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_series() {
        implements_auto_traits::<Series>();
    }

    #[test]
    fn should_keep_rank_zero_when_a_rank_agrees_within_the_less_precise_decimals() {
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let rounded = fact(&filing, "Revenues", "68000000");
        let mut exact = fact(&filing, "Revenues", "68106000");
        exact.decimals = Some(Decimals::Finite(0));
        exact.copy_rank = 1;
        let facts = vec![rounded.clone(), exact];

        let expected_result = Copies {
            facts: vec![&rounded],
            conflicts: Vec::new(),
        };

        let result = more_precise(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_every_rank_as_a_conflict_when_a_rank_disagrees_with_rank_zero() {
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let quarter = fact(&filing, "Revenues", "94930000000");
        let mut stub = fact(&filing, "Revenues", "61000000000");
        stub.copy_rank = 1;
        let facts = vec![stub.clone(), quarter.clone()];

        let expected_result = Copies {
            facts: Vec::new(),
            conflicts: vec![vec![&quarter, &stub]],
        };

        let result = more_precise(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_compare_with_rank_zero_only_when_two_later_ranks_disagree_with_each_other() {
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut millions = fact(&filing, "Revenues", "68000000");
        millions.decimals = Some(Decimals::Finite(-6));
        let mut low = fact(&filing, "Revenues", "67600000");
        low.decimals = Some(Decimals::Finite(0));
        low.copy_rank = 1;
        let mut high = fact(&filing, "Revenues", "68400000");
        high.decimals = Some(Decimals::Finite(0));
        high.copy_rank = 2;
        let facts = vec![millions.clone(), low, high];

        let expected_result = Copies {
            facts: vec![&millions],
            conflicts: Vec::new(),
        };

        let result = more_precise(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_conflict_when_a_rank_without_decimals_differs_from_rank_zero() {
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let mut first = fact(&filing, "Revenues", "68000000");
        first.decimals = None;
        let mut second = fact(&filing, "Revenues", "68106000");
        second.decimals = None;
        second.copy_rank = 1;
        let facts = vec![first.clone(), second.clone()];

        let expected_result = Copies {
            facts: Vec::new(),
            conflicts: vec![vec![&first, &second]],
        };

        let result = more_precise(&facts);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_value_of_the_latest_filed_filing_when_two_filings_report_a_period() {
        let older = candidate(&source("0000320193-24-000123", date(2024, 11, 1)), "100");
        let newer = candidate(&source("0000320193-25-000010", date(2025, 10, 31)), "110");
        let candidates = vec![newer, older];

        let expected_result = vec![BigDecimal::from(110)];

        let result: Vec<BigDecimal> = newest_filing(&candidates)
            .facts
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_later_acceptance_when_two_filings_are_filed_on_one_day() {
        let mut early = source("0000320193-25-000020", date(2025, 10, 31));
        early.accepted = Some(moment(2025, 10, 31, 14));
        let mut late = source("0000320193-25-000010", date(2025, 10, 31));
        late.accepted = Some(moment(2025, 10, 31, 20));
        let candidates = vec![candidate(&late, "110"), candidate(&early, "100")];

        let expected_result = vec![BigDecimal::from(110)];

        let result: Vec<BigDecimal> = newest_filing(&candidates)
            .facts
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_higher_filing_id_when_two_filings_have_one_filed_date_and_no_acceptance() {
        let low = source("0000320193-25-000010", date(2025, 10, 31));
        let high = source("0000320193-25-000020", date(2025, 10, 31));
        let candidates = vec![candidate(&high, "110"), candidate(&low, "100")];

        let expected_result = vec![BigDecimal::from(110)];

        let result: Vec<BigDecimal> = newest_filing(&candidates)
            .facts
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_fact_when_the_newest_candidate_is_blocked() {
        let older = candidate(&source("0000320193-24-000123", date(2024, 11, 1)), "100");
        let mut newer = candidate(&source("0000320193-25-000010", date(2025, 10, 31)), "110");
        newer.value = CandidateValue::Blocked;
        let key = newer.series_key();
        let candidates = vec![older, newer];

        let expected_result = Series {
            facts: Vec::new(),
            blocked: vec![key],
        };

        let result = newest_filing(&candidates);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_newer_value_when_an_older_candidate_is_blocked() {
        let mut older = candidate(&source("0000320193-24-000123", date(2024, 11, 1)), "100");
        older.value = CandidateValue::Blocked;
        let newer = candidate(&source("0000320193-25-000010", date(2025, 10, 31)), "110");
        let candidates = vec![older, newer];

        let expected_result = vec![(BigDecimal::from(110), false)];

        let result: Vec<(BigDecimal, bool)> = newest_filing(&candidates)
            .facts
            .into_iter()
            .map(|fact| (fact.value, fact.restated))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_mark_the_fact_restated_when_an_older_filing_reports_another_value() {
        let older = candidate(
            &source("0000320193-24-000123", date(2024, 11, 1)),
            "100000000",
        );
        let newer = candidate(
            &source("0000320193-25-000010", date(2025, 10, 31)),
            "110000000",
        );
        let candidates = vec![older, newer];

        let expected_result = vec![true];

        let result: Vec<bool> = newest_filing(&candidates)
            .facts
            .into_iter()
            .map(|fact| fact.restated)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_mark_the_fact_restated_when_an_older_value_agrees_within_its_precision() {
        let older = candidate(
            &source("0000320193-24-000123", date(2024, 11, 1)),
            "68000000",
        );
        let mut newer = candidate(
            &source("0000320193-25-000010", date(2025, 10, 31)),
            "68106000",
        );
        newer.value = CandidateValue::Reported {
            value: BigDecimal::from(68_106_000),
            decimals: Some(Decimals::Finite(0)),
            confidence: Confidence::Exact,
        };
        let candidates = vec![older, newer];

        let expected_result = vec![false];

        let result: Vec<bool> = newest_filing(&candidates)
            .facts
            .into_iter()
            .map(|fact| fact.restated)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_match_two_filings_when_their_periods_differ_only_in_the_exact_start() {
        let mut older = candidate(&source("0000320193-24-000123", date(2024, 11, 1)), "100");
        older.period = Period::Duration(
            Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
                .expect("Given an end after the start, the creation should always succeed"),
        );
        let newer = candidate(&source("0000320193-25-000010", date(2025, 10, 31)), "100");
        let candidates = vec![older, newer];

        let expected_result = 1;

        let result = newest_filing(&candidates).facts.len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_two_series_when_the_lengths_in_quarters_differ() {
        let filing = source("0000320193-24-000123", date(2024, 11, 1));
        let year = candidate(&filing, "400");
        let mut quarter = candidate(&filing, "100");
        quarter.period = Period::Duration(Duration::from_end_and_quarters(date(2024, 9, 28), 1));
        let candidates = vec![year, quarter];

        let expected_result = vec![1, 4];

        let result: Vec<u16> = newest_filing(&candidates)
            .facts
            .into_iter()
            .map(|fact| fact.period.quarters())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_ignore_a_filing_accepted_after_the_read_time_when_reading_as_of_a_time() {
        let mut older_source = source("0000320193-24-000123", date(2024, 11, 1));
        older_source.accepted = Some(moment(2024, 11, 1, 20));
        let mut newer_source = source("0000320193-25-000010", date(2025, 10, 31));
        newer_source.accepted = Some(moment(2025, 10, 31, 20));
        let candidates = vec![
            candidate(&older_source, "100000000"),
            candidate(&newer_source, "110000000"),
        ];
        let as_of = moment(2025, 10, 31, 12);

        let expected_result = vec![BigDecimal::from(100_000_000)];

        let result: Vec<BigDecimal> = newest_filing_as_of(&candidates, as_of)
            .facts
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_include_a_filing_without_acceptance_when_it_is_filed_on_the_read_date() {
        let older = source("0000320193-24-000123", date(2024, 11, 1));
        let newer = source("0000320193-25-000010", date(2025, 10, 31));
        let candidates = vec![
            candidate(&older, "100000000"),
            candidate(&newer, "110000000"),
        ];
        let as_of = moment(2025, 10, 31, 12);

        let expected_result = vec![BigDecimal::from(110_000_000)];

        let result: Vec<BigDecimal> = newest_filing_as_of(&candidates, as_of)
            .facts
            .into_iter()
            .map(|fact| fact.value)
            .collect();

        assert_eq!(result, expected_result);
    }
}
