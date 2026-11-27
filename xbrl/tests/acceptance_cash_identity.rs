//! # Acceptance Test A4: Cash Identity
//!
//! Runs the checks of the roll-up tree on one fixture 10-K, after the rules of A1
//! (`acceptance_roll_ups.rs`), and lists the failed checks of the closed group
//! `net-change-in-cash`: the three cash flows and the exchange rate effect sum to the change in
//! cash.
//!
//! `UnitedHealth` reports cash held for sale in its change in cash for 2024 and 2025, so its 10-K for
//! fiscal year 2025 fails the check for both years. Apple fails no check.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use jiff::civil::Date;
use pretty_assertions::assert_eq;
use xbrl::core::elements::GroupName;
use xbrl::core::elements::statement::cash_flow::CashFlowGroup;
use xbrl::core::finding::{Finding, RollUpCheck};
use xbrl::core::roll_up_check::check_roll_ups;
use xbrl::tests::fixtures::data::resolved::{derived_values, resolve_fixture_filings};
use xbrl::tests::fixtures::sample_fact::date;

/// Returns the start and the end of each failed check of `net-change-in-cash` in `findings`.
fn cash_mismatches(findings: Vec<Finding>) -> Vec<(Option<Date>, Date)> {
    findings
        .into_iter()
        .filter_map(|finding| match finding {
            Finding::RollUpMismatch {
                period,
                reason: RollUpCheck::Group(GroupName::CashFlow(CashFlowGroup::NetChangeInCash)),
                ..
            } => Some((period.start(), period.end())),
            _ => None,
        })
        .collect()
}

#[test]
fn should_fail_the_cash_identity_of_2024_and_2025_for_unitedhealth_10_k_2025_fy() {
    let series = resolve_fixture_filings(&["0000731766-26-000062"]).series();
    let added = derived_values(&series);

    let expected_result = vec![
        (Some(date(2024, 1, 1)), date(2024, 12, 31)),
        (Some(date(2025, 1, 1)), date(2025, 12, 31)),
    ];

    let result = cash_mismatches(check_roll_ups(&series, &added));

    assert_eq!(result, expected_result);
}

#[test]
fn should_fail_no_cash_identity_for_apple_10_k_2025_fy() {
    let series = resolve_fixture_filings(&["0000320193-25-000079"]).series();
    let added = derived_values(&series);

    let expected_result: Vec<(Option<Date>, Date)> = Vec::new();

    let result = cash_mismatches(check_roll_ups(&series, &added));

    assert_eq!(result, expected_result);
}
