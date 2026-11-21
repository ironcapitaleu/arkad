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

use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use xbrl::core::closed_group::derive_from_closed_groups;
use xbrl::core::finding::Finding;
use xbrl::core::period_identity::derive_q4;
use xbrl::core::roll_up_check::check_roll_ups;
use xbrl::tests::fixtures::data::resolved::resolve_fixture_filings;
use xbrl::tests::fixtures::sample_fact::date;

/// The name of the closed group of the change in cash in findings.
const NET_CHANGE_IN_CASH: &str = "net-change-in-cash";

/// Returns the start and the end of each failed check of `net-change-in-cash` in `findings`.
fn cash_mismatches(findings: Vec<Finding>) -> Vec<(Option<NaiveDate>, NaiveDate)> {
    findings
        .into_iter()
        .filter_map(|finding| match finding {
            Finding::RollUpMismatch { period, reason, .. }
                if reason.as_str() == NET_CHANGE_IN_CASH =>
            {
                Some((period.start(), period.end()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn should_fail_the_cash_identity_of_2024_and_2025_for_unitedhealth_10_k_2025_fy() {
    let series = resolve_fixture_filings(&["0000731766-26-000062"]).series();
    let mut added = derive_q4(&series);
    let derived = derive_from_closed_groups(&series, &added);
    added.extend(derived);

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
    let mut added = derive_q4(&series);
    let derived = derive_from_closed_groups(&series, &added);
    added.extend(derived);

    let expected_result: Vec<(Option<NaiveDate>, NaiveDate)> = Vec::new();

    let result = cash_mismatches(check_roll_ups(&series, &added));

    assert_eq!(result, expected_result);
}
