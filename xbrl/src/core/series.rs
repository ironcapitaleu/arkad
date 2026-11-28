//! # Series
//!
//! Provides [`standardize_series`], which runs the rules that need every filing of one entity, and
//! its output [`SeriesOutcome`].
//!
//! The rules run in the order that [`standardize_series`] lists: [`newest_filing`], [`derive_q4`],
//! [`derive_from_closed_groups`], the checks, the check of the required concepts, then
//! chronology. Values with dimensions pass
//! [`newest_filing`] and reach the output. The later rules read only values without dimensions. No rule here builds the
//! segment reconciliation, so the output holds no line of it.

use super::candidate::Candidate;
use super::chronology::check_chronology;
use super::closed_group::derive_from_closed_groups;
use super::dedup::{Series, newest_filing};
use super::elements::CanonicalElementKind;
use super::filing::Filing;
use super::finding::Finding;
use super::period_identity::derive_q4;
use super::required_check::check_required;
use super::roll_up_check::check_roll_ups;
use super::standardized::StandardizedFact;
use super::sum_check::check_all_sums;

/// The standardized facts and the findings of one entity after [`standardize_series`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesOutcome {
    /// The facts of `core/dedup/newest-filing@1`, the fourth quarters, and the roll-up values,
    /// sorted by series key, with the blocked series keys.
    pub series: Series,
    /// The findings of [`check_roll_ups`], then of [`check_all_sums`], then of
    /// [`check_required`], then of [`check_chronology`], which end with the conflicts.
    pub findings: Vec<Finding>,
}

/// Runs [`newest_filing`], [`derive_q4`], [`derive_from_closed_groups`], the checks, and
/// chronology on the candidates of one entity.
///
/// The function calls, in this order:
///
/// 1. [`newest_filing`] on `candidates`, which gives the series.
/// 2. [`derive_q4`] on the series, which gives the fourth quarters.
/// 3. [`derive_from_closed_groups`] on the series and the fourth quarters, which gives the roll-up
///    values.
/// 4. `added` is the fourth quarters, then the roll-up values.
/// 5. [`check_roll_ups`] with `added`.
/// 6. [`check_all_sums`] with the same `added`.
/// 7. [`check_required`] with the same `added`, `candidates`, `filings`, and `required`.
/// 8. [`check_chronology`] on the series with `added` merged in, with `filings` and with
///    `conflicts` as the findings of dedup.
///
/// `candidates` holds the candidates of every filing of the entity. `filings` holds the filings of
/// the entity, for the forms and the fiscal year end. `conflicts` holds the
/// [`Finding::Conflict`] findings of the filings of `candidates`. `required` holds the required
/// standard concepts of the plug-in, such as [`UsGaap::required_concepts`].
///
/// [`UsGaap::required_concepts`]: crate::us_gaap::UsGaap::required_concepts
#[must_use]
pub fn standardize_series(
    candidates: &[Candidate],
    filings: &[Filing],
    conflicts: Vec<Finding>,
    required: &[CanonicalElementKind],
) -> SeriesOutcome {
    let series = newest_filing(candidates);
    let mut added = derive_q4(&series);
    let roll_ups = derive_from_closed_groups(&series, &added);
    added.extend(roll_ups);
    let mut findings = check_roll_ups(&series, &added);
    findings.extend(check_all_sums(&series, &added));
    findings.extend(check_required(
        &series, &added, candidates, filings, required,
    ));
    let chronology = check_chronology(merged(series, added), filings, conflicts);
    findings.extend(chronology.findings);
    SeriesOutcome {
        series: chronology.series,
        findings,
    }
}

/// Returns `series` with `added` in its facts, sorted by series key.
fn merged(series: Series, added: Vec<StandardizedFact>) -> Series {
    let mut facts = series.facts;
    facts.extend(added);
    facts.sort_by_cached_key(StandardizedFact::series_key);
    Series {
        facts,
        blocked: series.blocked,
    }
}

#[cfg(test)]
mod tests {
    use bigdecimal::BigDecimal;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::candidate::CandidateValue;
    use crate::core::elements::{
        BalanceSheetConcept, CanonicalElementKind, Concept, DebtMaturityConcept,
        IncomeStatementConcept,
    };
    use crate::core::fact::{Dimensions, FilingSource};
    use crate::core::period::{Duration, Period};
    use crate::core::rule::RuleId;
    use crate::core::segment::ReconciliationLine;
    use crate::tests::fixtures::sample_fact::{candidate, date, filing, source};

    /// The concept, the period, and the rule of a standardized fact.
    type Summary = (CanonicalElementKind, String, String);

    /// Returns the filing `0000320193-24-000123`, filed on 2024-11-01.
    fn annual_report() -> FilingSource {
        source("0000320193-24-000123", date(2024, 11, 1))
    }

    /// Returns the fiscal year from 2023-10-01 to 2024-09-28.
    fn fiscal_year() -> Period {
        Period::Duration(
            Duration::from_dates(date(2023, 10, 1), date(2024, 9, 28))
                .expect("Given an end after the start, the creation should always succeed"),
        )
    }

    /// Returns the nine months from 2023-10-01 to 2024-06-29.
    fn nine_months() -> Period {
        Period::Duration(
            Duration::from_dates(date(2023, 10, 1), date(2024, 6, 29))
                .expect("Given an end after the start, the creation should always succeed"),
        )
    }

    /// Returns the fiscal year end 2024-09-28.
    const fn year_end() -> Period {
        Period::Instant {
            end: date(2024, 9, 28),
        }
    }

    /// Returns the candidate `concept` = `amount` at `period` from `filing`.
    fn value(
        filing: &FilingSource,
        concept: impl Into<CanonicalElementKind>,
        period: Period,
        amount: &str,
    ) -> Candidate {
        Candidate {
            concept: concept.into(),
            period,
            ..candidate(filing, amount)
        }
    }

    /// Returns the candidate `concept` = `amount` at [`year_end`] from [`annual_report`].
    fn at_year_end(concept: impl Into<CanonicalElementKind>, amount: &str) -> Candidate {
        value(&annual_report(), concept, year_end(), amount)
    }

    /// Returns the revenue = `amount` of the fiscal year from [`annual_report`], with `pairs` as its
    /// dimensions.
    fn revenue_in(pairs: &[(&str, &str)], amount: &str) -> Candidate {
        Candidate {
            dimensions: Dimensions::new(pairs.iter().copied()).expect(
                "Given hardcoded axis and member pairs, the creation should always succeed",
            ),
            ..value(
                &annual_report(),
                IncomeStatementConcept::Revenue.element(),
                fiscal_year(),
                amount,
            )
        }
    }

    /// Returns a balance sheet at [`year_end`] whose `Liabilities` is unknown and adds up to 7,468.
    fn balance_sheet_without_liabilities() -> Vec<Candidate> {
        vec![
            at_year_end(BalanceSheetConcept::LiabilitiesAndEquity.element(), "7216"),
            at_year_end(BalanceSheetConcept::Equity.element(), "-252"),
            at_year_end(
                BalanceSheetConcept::EquityAttributableToParent.element(),
                "-252",
            ),
        ]
    }

    /// Returns a ladder at [`year_end`] with the total 1,000 and six buckets of 100, so the ladder
    /// adds up to 600 and fails the check.
    fn ladder_that_does_not_add_up() -> Vec<Candidate> {
        DebtMaturityConcept::ALL
            .iter()
            .map(|concept| match concept {
                DebtMaturityConcept::LongTermDebtIncludingCurrentMaturities => {
                    at_year_end(concept.element(), "1000")
                }
                _ => at_year_end(concept.element(), "100"),
            })
            .collect()
    }

    /// Returns the fourth quarter from 2024-06-30 to 2024-09-28.
    fn fourth_quarter() -> Period {
        Period::Duration(
            Duration::from_dates(date(2024, 6, 30), date(2024, 9, 28))
                .expect("Given an end after the start, the creation should always succeed"),
        )
    }

    /// Returns the revenue of the fiscal year, 400, and of the nine months, 300, so the fourth
    /// quarter is 100.
    fn revenue_of_the_year_and_nine_months() -> Vec<Candidate> {
        vec![
            value(
                &annual_report(),
                IncomeStatementConcept::Revenue.element(),
                fiscal_year(),
                "400",
            ),
            value(
                &annual_report(),
                IncomeStatementConcept::Revenue.element(),
                nine_months(),
                "300",
            ),
        ]
    }

    /// Returns the [`Summary`] of each fact of `outcome` whose rule starts with `core/`.
    fn derived_summaries(outcome: &SeriesOutcome) -> Vec<Summary> {
        outcome
            .series
            .facts
            .iter()
            .filter(|fact| fact.rule.as_str().starts_with("core/"))
            .map(|fact| (fact.concept, fact.period.to_string(), fact.rule.to_string()))
            .collect()
    }

    /// Returns the name of the variant of each finding in `findings`.
    fn kinds(findings: &[Finding]) -> Vec<&'static str> {
        findings
            .iter()
            .map(|finding| match finding {
                Finding::UndeclaredFamily { .. } => "UndeclaredFamily",
                Finding::UnmappedRow { .. } => "UnmappedRow",
                Finding::Conflict { .. } => "Conflict",
                Finding::Gap { .. } => "Gap",
                Finding::MissingRequired { .. } => "MissingRequired",
                Finding::Overlap { .. } => "Overlap",
                Finding::Restatement { .. } => "Restatement",
                Finding::SuspiciousPeriod { .. } => "SuspiciousPeriod",
                Finding::RollUpMismatch { .. } => "RollUpMismatch",
                Finding::NegativeOther { .. } => "NegativeOther",
                Finding::MixedStarts { .. } => "MixedStarts",
                Finding::MixedScopes { .. } => "MixedScopes",
                Finding::SumMismatch { .. } => "SumMismatch",
            })
            .collect()
    }

    /// Returns a conflict of the revenue of [`annual_report`] between the ranks 0 and 1.
    fn revenue_conflict() -> Finding {
        let revenue = candidate(&annual_report(), "100");
        Finding::Conflict {
            entity: revenue.entity,
            key: revenue
                .inputs
                .first()
                .cloned()
                .expect("Given the revenue builder, the candidate should always have one input"),
            ranks: vec![1],
            standard_concept: Some(IncomeStatementConcept::Revenue.element()),
        }
    }

    #[test]
    fn should_give_the_fourth_quarter_and_the_roll_up_value_when_the_series_holds_their_inputs() {
        let mut candidates = vec![
            value(
                &annual_report(),
                IncomeStatementConcept::Revenue.element(),
                fiscal_year(),
                "400",
            ),
            value(
                &annual_report(),
                IncomeStatementConcept::Revenue.element(),
                nine_months(),
                "300",
            ),
        ];
        candidates.extend(balance_sheet_without_liabilities());
        let filings: Vec<Filing> = Vec::new();
        let conflicts = Vec::new();

        let expected_result = vec![
            (
                BalanceSheetConcept::Liabilities.element(),
                "2024-09-28".to_string(),
                "core/roll-up/liabilities-and-equity@1".to_string(),
            ),
            (
                IncomeStatementConcept::Revenue.element(),
                "2024-06-30 to 2024-09-28".to_string(),
                "core/period/q4-from-fy@1".to_string(),
            ),
        ];

        let result = derived_summaries(&standardize_series(&candidates, &filings, conflicts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_roll_up_mismatch_and_a_sum_mismatch_when_a_group_and_a_ladder_do_not_add_up()
    {
        let mut candidates = vec![at_year_end(
            BalanceSheetConcept::Liabilities.element(),
            "7000",
        )];
        candidates.extend(balance_sheet_without_liabilities());
        candidates.extend(ladder_that_does_not_add_up());
        let filings: Vec<Filing> = Vec::new();
        let conflicts = Vec::new();

        let expected_result = vec!["RollUpMismatch", "SumMismatch"];

        let result = kinds(&standardize_series(&candidates, &filings, conflicts, &[]).findings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_fact_with_a_role_rule_when_a_filing_reports_segment_values() {
        let candidates = vec![
            revenue_in(&[], "190"),
            revenue_in(&[("BusinessSegments", "Cloud")], "100"),
            revenue_in(&[("BusinessSegments", "Devices")], "100"),
            revenue_in(&[("ConsolidationItems", "IntersegmentElimination")], "-10"),
            revenue_in(&[("ConsolidationItems", "CorporateNonSegment")], "5"),
        ];
        let any = candidate(&annual_report(), "0");
        let role_rules: Vec<RuleId> = [
            ReconciliationLine::Elimination(any.clone()),
            ReconciliationLine::Corporate(any),
            ReconciliationLine::Unreconciled(BigDecimal::from(0)),
        ]
        .iter()
        .filter_map(ReconciliationLine::role_rule)
        .collect();
        let filings: Vec<Filing> = Vec::new();
        let conflicts = Vec::new();

        let expected_result: Vec<String> = Vec::new();

        let result: Vec<String> = standardize_series(&candidates, &filings, conflicts, &[])
            .series
            .facts
            .iter()
            .filter(|fact| role_rules.contains(&fact.rule))
            .map(|fact| fact.rule.to_string())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_conflicts_after_the_findings_of_chronology_when_dedup_reports_a_conflict() {
        let candidates = vec![
            candidate(
                &source("0000320193-23-000106", date(2023, 11, 3)),
                "390000000",
            ),
            candidate(&annual_report(), "391000000"),
        ];
        let filings: Vec<Filing> = Vec::new();
        let conflicts = vec![revenue_conflict()];

        let expected_result = vec!["Restatement", "Conflict"];

        let result = kinds(&standardize_series(&candidates, &filings, conflicts, &[]).findings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_dimensions_of_a_segment_value_when_the_series_holds_it() {
        let segment = revenue_in(&[("BusinessSegments", "Cloud")], "100");
        let candidates = vec![segment.clone()];
        let filings: Vec<Filing> = Vec::new();
        let conflicts = Vec::new();

        let expected_result = vec![segment.dimensions];

        let result: Vec<Dimensions> = standardize_series(&candidates, &filings, conflicts, &[])
            .series
            .facts
            .into_iter()
            .map(|fact| fact.dimensions)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_a_roll_up_value_from_a_fourth_quarter_when_the_group_needs_it() {
        let mut candidates = revenue_of_the_year_and_nine_months();
        candidates.push(value(
            &annual_report(),
            IncomeStatementConcept::GrossProfit.element(),
            fourth_quarter(),
            "40",
        ));
        let filings: Vec<Filing> = Vec::new();
        let conflicts = Vec::new();

        let expected_result = vec![
            (
                IncomeStatementConcept::Revenue.element(),
                "2024-06-30 to 2024-09-28".to_string(),
                "core/period/q4-from-fy@1".to_string(),
            ),
            (
                IncomeStatementConcept::CostOfRevenue.element(),
                "2024-06-30 to 2024-09-28".to_string(),
                "core/roll-up/gross-profit@1".to_string(),
            ),
        ];

        let result = derived_summaries(&standardize_series(&candidates, &filings, conflicts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_roll_up_mismatch_when_a_fourth_quarter_breaks_a_group() {
        let mut candidates = revenue_of_the_year_and_nine_months();
        candidates.extend([
            value(
                &annual_report(),
                IncomeStatementConcept::GrossProfit.element(),
                fourth_quarter(),
                "40",
            ),
            value(
                &annual_report(),
                IncomeStatementConcept::CostOfRevenue.element(),
                fourth_quarter(),
                "20",
            ),
        ]);
        let filings: Vec<Filing> = Vec::new();
        let conflicts = Vec::new();

        let expected_result = vec!["RollUpMismatch"];

        let result = kinds(&standardize_series(&candidates, &filings, conflicts, &[]).findings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_missing_required_before_chronology_when_a_required_concept_has_no_value() {
        let candidates: Vec<Candidate> = Vec::new();
        let filings = vec![filing("10-K", date(2024, 9, 28), "0928")];
        let conflicts = vec![revenue_conflict()];
        let required = [BalanceSheetConcept::Assets.element()];

        let expected_result = vec!["MissingRequired", "Conflict"];

        let result =
            kinds(&standardize_series(&candidates, &filings, conflicts, &required).findings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_a_blocked_series_key_when_the_newest_candidate_is_blocked() {
        let blocked = Candidate {
            value: CandidateValue::Blocked,
            ..candidate(&annual_report(), "391")
        };
        let candidates = vec![blocked.clone()];
        let filings: Vec<Filing> = Vec::new();
        let conflicts = Vec::new();

        let expected_result = vec![blocked.series_key()];

        let result = standardize_series(&candidates, &filings, conflicts, &[])
            .series
            .blocked;

        assert_eq!(result, expected_result);
    }
}
