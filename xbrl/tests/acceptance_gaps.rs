//! # Acceptance Test A7: Gaps
//!
//! Runs the gap check on the series of one company over several fixture filings, after the rules of
//! one filing and `core/dedup/newest-filing@1`.
//!
//! - Supermicro filed no 10-K for fiscal year 2018. Its 10-K for fiscal year 2019 reports fiscal
//!   year 2018 as a comparative, so the series holds a value for every year, and the check reports
//!   no gap. The check reads values, not filings, so it does not see the missing 10-K. The 10-Ks of
//!   fiscal years 2013 and 2017 alone leave fiscal year 2014 without a value, and the check reports
//!   that gap.
//! - Deckers moved its fiscal year end from 31 December to 31 March with its 10-QT of 2014. The
//!   fixture set holds no filing between the 10-QT and fiscal year 2025, so the check reports a gap
//!   for each quarter between them. The gaps in the 12 months after 31 December 2013, the last
//!   period of report under the old fiscal year end, get the reason [`GapReason::FiscalYearChange`].
//!   The test lists the gaps of net income with a reason.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use xbrl::core::chronology::check_chronology;
use xbrl::core::elements::{CanonicalElementKind, IncomeStatementConcept};
use xbrl::core::finding::{Finding, GapReason};
use xbrl::core::fiscal_period::FiscalPeriod;
use xbrl::tests::fixtures::data::resolved::resolve_fixture_filings;
use xbrl::tests::fixtures::sample_fact::date;

/// The end, the fiscal period, the length in quarters, and the reason of a gap.
type GapSummary = (NaiveDate, Option<String>, u16, Option<GapReason>);

/// Returns the [`GapSummary`] of each gap of `concept` in `findings`.
fn gaps_of(findings: Vec<Finding>, concept: CanonicalElementKind) -> Vec<GapSummary> {
    findings
        .into_iter()
        .filter_map(|finding| match finding {
            Finding::Gap {
                concept: gap_concept,
                period_end,
                fiscal_period,
                quarters,
                reason,
                ..
            } if gap_concept == concept => Some((
                period_end,
                fiscal_period.as_ref().map(FiscalPeriod::to_string),
                quarters,
                reason,
            )),
            _ => None,
        })
        .collect()
}

#[test]
fn should_report_no_gap_when_the_supermicro_10_k_2019_fy_reports_fiscal_year_2018() {
    let resolved = resolve_fixture_filings(&["0001375365-19-000039", "0001375365-19-000079"]);

    let expected_result: Vec<Finding> = Vec::new();

    let result: Vec<Finding> = check_chronology(resolved.series(), &resolved.filings, Vec::new())
        .findings
        .into_iter()
        .filter(|finding| matches!(finding, Finding::Gap { .. }))
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_report_the_gap_of_fiscal_year_2014_when_only_the_supermicro_10_ks_2013_and_2017_fy_report_net_income()
 {
    let resolved = resolve_fixture_filings(&["0001445305-13-002262", "0001375365-19-000039"]);
    let concept = IncomeStatementConcept::NetIncome.element();

    let expected_result = vec![(date(2014, 6, 30), Some("FY".to_string()), 4, None)];

    let result = gaps_of(
        check_chronology(resolved.series(), &resolved.filings, Vec::new()).findings,
        concept,
    );

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_the_reason_fiscal_year_change_when_deckers_moves_its_fiscal_year_end() {
    let resolved = resolve_fixture_filings(&[
        "0001445305-14-000820",
        "0000910521-14-000013",
        "0000910521-24-000042",
        "0000910521-24-000059",
        "0000910521-25-000005",
        "0000910521-25-000017",
    ]);
    let concept = IncomeStatementConcept::NetIncome.element();

    let expected_result = vec![
        (
            date(2014, 6, 30),
            Some("Q1".to_string()),
            1,
            Some(GapReason::FiscalYearChange),
        ),
        (
            date(2014, 9, 30),
            Some("Q2".to_string()),
            1,
            Some(GapReason::FiscalYearChange),
        ),
        (
            date(2014, 12, 31),
            Some("Q3".to_string()),
            1,
            Some(GapReason::FiscalYearChange),
        ),
    ];

    let result: Vec<GapSummary> = gaps_of(
        check_chronology(resolved.series(), &resolved.filings, Vec::new()).findings,
        concept,
    )
    .into_iter()
    .filter(|(.., reason)| reason.is_some())
    .collect();

    assert_eq!(result, expected_result);
}
