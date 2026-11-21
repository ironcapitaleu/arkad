//! # Roll-Up Real Filing Integration Tests
//!
//! Runs the derivation from a closed group and the checks of the roll-up tree on the standardized
//! facts of one real filing, the 10-Q `0001628280-26-068246`, as
//! [`xbrl::adapter::edgar::notes::parse`] and [`xbrl::adapter::edgar::instance::parse`] give them.
//!
//! The filing reports no `Liabilities` and no `AssetsNoncurrent` on its balance sheets of
//! 2026-05-31 and 2026-09-30. It reports no temporary equity and no noncontrolling interest, so both
//! count as zero. Each checked group and the balance sheet identity hold, so the checks report no
//! mismatch.
//!
//! Both parsers give each duration its exact start. The filing reports its cash flow lines only
//! for four months from 1 June, and net income and D&A for the quarter from 1 July as well. So
//! `operating-cash-flow` starts from the quarter and holds four-month lines, and it reports
//! `MixedStarts` in each quarter. `net-change-in-cash` holds only four-month values, and the income
//! statement holds only quarters, so their groups keep their checks.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use xbrl::core::alias::resolve_aliases;
use xbrl::core::closed_group::derive_from_closed_groups;
use xbrl::core::dedup::{Series, more_precise, newest_filing};
use xbrl::core::elements::{
    BalanceSheetConcept, CanonicalElementKind, CashFlowConcept, IncomeStatementConcept,
};
use xbrl::core::fact::AsIsFact;
use xbrl::core::finding::Finding;
use xbrl::core::period_identity::derive_q4;
use xbrl::core::roll_up_check::check_roll_ups;
use xbrl::core::taxonomy::registry::{Plugin, Registry};
use xbrl::tests::fixtures::data::instance::parse_instance_fixture;
use xbrl::tests::fixtures::data::notes::parse_notes_fixture;
use xbrl::tests::fixtures::sample_fact::date;
use xbrl::us_gaap::UsGaap;

/// The concept, the period, the value, and the rule of a derived fact.
type Summary = (CanonicalElementKind, String, BigDecimal, String);

/// Runs the dedup rules, tiers 1 and 2 with [`UsGaap`], `core/period/q4-from-fy@1`, and the
/// derivation from a closed group on `facts`, and returns the derived facts of the last step.
fn roll_ups(facts: &[AsIsFact]) -> Vec<Summary> {
    let plugin = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&plugin)])
        .expect("Given one plug-in with distinct families, the creation should always succeed");
    let series = newest_filing(&resolve_aliases(&more_precise(facts), &registry).candidates);
    let fourth_quarters = derive_q4(&series, &[]);
    derive_from_closed_groups(&series, &fourth_quarters)
        .into_iter()
        .map(|fact| {
            (
                fact.concept,
                fact.period.to_string(),
                fact.value,
                fact.rule.to_string(),
            )
        })
        .collect()
}

/// The derived facts of the fixture filing, in millions of USD: `NonCurrentAssets` from `assets`,
/// then `Liabilities` from `liabilities-and-equity`.
fn expected_roll_ups() -> Vec<Summary> {
    vec![
        (
            BalanceSheetConcept::NonCurrentAssets.element(),
            "2026-05-31".to_string(),
            BigDecimal::from(5_378_000_000_i64),
            "core/roll-up/assets@1".to_string(),
        ),
        (
            BalanceSheetConcept::NonCurrentAssets.element(),
            "2026-09-30".to_string(),
            BigDecimal::from(5_370_000_000_i64),
            "core/roll-up/assets@1".to_string(),
        ),
        (
            BalanceSheetConcept::Liabilities.element(),
            "2026-05-31".to_string(),
            BigDecimal::from(7_381_000_000_i64),
            "core/roll-up/liabilities-and-equity@1".to_string(),
        ),
        (
            BalanceSheetConcept::Liabilities.element(),
            "2026-09-30".to_string(),
            BigDecimal::from(7_468_000_000_i64),
            "core/roll-up/liabilities-and-equity@1".to_string(),
        ),
    ]
}

#[test]
fn should_derive_liabilities_and_non_current_assets_when_the_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    let expected_result = expected_roll_ups();

    let result = roll_ups(&facts);

    assert_eq!(result, expected_result);
}

#[test]
fn should_derive_liabilities_and_non_current_assets_when_the_rules_run_on_the_instance_facts() {
    let facts = parse_instance_fixture().facts;

    let expected_result = expected_roll_ups();

    let result = roll_ups(&facts);

    assert_eq!(result, expected_result);
}

/// Runs the dedup rules and tiers 1 and 2 with [`UsGaap`] on `facts`, and returns the series.
fn series_of(facts: &[AsIsFact]) -> Series {
    let plugin = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&plugin)])
        .expect("Given one plug-in with distinct families, the creation should always succeed");
    newest_filing(&resolve_aliases(&more_precise(facts), &registry).candidates)
}

/// Runs the rules of [`roll_ups`], then the checks of the roll-up tree, and returns the findings.
fn findings(facts: &[AsIsFact]) -> Vec<Finding> {
    findings_of(&series_of(facts))
}

/// Runs `core/period/q4-from-fy@1`, the derivation from a closed group, and the checks of the
/// roll-up tree on `series`, and returns the findings.
fn findings_of(series: &Series) -> Vec<Finding> {
    let mut added = derive_q4(series, &[]);
    let derived = derive_from_closed_groups(series, &added);
    added.extend(derived);
    check_roll_ups(series, &added)
}

/// The kind of a finding, the name of its group, and the end of its period.
type FindingSummary = (&'static str, String, NaiveDate);

/// Returns the [`FindingSummary`] of each roll-up finding in `findings`.
fn summaries(findings: Vec<Finding>) -> Vec<FindingSummary> {
    findings
        .into_iter()
        .filter_map(|finding| match finding {
            Finding::MixedStarts { period, reason, .. } => {
                Some(("MixedStarts", reason.to_string(), period.end()))
            }
            Finding::RollUpMismatch { period, reason, .. } => {
                Some(("RollUpMismatch", reason.to_string(), period.end()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn should_report_mixed_starts_of_operating_cash_flow_in_each_quarter_when_the_checks_run_on_the_notes_facts()
 {
    let facts = parse_notes_fixture().facts;

    let expected_result = vec![
        (
            "MixedStarts",
            "operating-cash-flow".to_string(),
            date(2025, 9, 30),
        ),
        (
            "MixedStarts",
            "operating-cash-flow".to_string(),
            date(2026, 9, 30),
        ),
    ];

    let result = summaries(findings(&facts));

    assert_eq!(result, expected_result);
}

#[test]
fn should_report_mixed_starts_of_operating_cash_flow_in_each_quarter_when_the_checks_run_on_the_instance_facts()
 {
    let facts = parse_instance_fixture().facts;

    let expected_result = vec![
        (
            "MixedStarts",
            "operating-cash-flow".to_string(),
            date(2025, 9, 30),
        ),
        (
            "MixedStarts",
            "operating-cash-flow".to_string(),
            date(2026, 9, 30),
        ),
    ];

    let result = summaries(findings(&facts));

    assert_eq!(result, expected_result);
}

#[test]
fn should_keep_the_checks_of_net_change_in_cash_and_net_income_attribution_when_the_instance_quarter_mixes_starts()
 {
    let mut series = series_of(&parse_instance_fixture().facts);
    // Shifts two totals of the 2026 quarter, so each check that still runs fails.
    for fact in &mut series.facts {
        let shifted = [
            CashFlowConcept::NetChangeInCash.element(),
            IncomeStatementConcept::NetIncomeAttributableToParent.element(),
        ];
        if shifted.contains(&fact.concept)
            && fact.dimensions.is_empty()
            && fact.period.end() == date(2026, 9, 30)
            && fact.period.quarters() == 1
        {
            fact.value += BigDecimal::from(100_000_000);
        }
    }

    let expected_result = vec![
        (
            "MixedStarts",
            "operating-cash-flow".to_string(),
            date(2025, 9, 30),
        ),
        (
            "MixedStarts",
            "operating-cash-flow".to_string(),
            date(2026, 9, 30),
        ),
        (
            "RollUpMismatch",
            "net-income-attribution".to_string(),
            date(2026, 9, 30),
        ),
        (
            "RollUpMismatch",
            "net-change-in-cash".to_string(),
            date(2026, 9, 30),
        ),
    ];

    let result = summaries(findings_of(&series));

    assert_eq!(result, expected_result);
}
