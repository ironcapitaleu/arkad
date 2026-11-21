//! # Acceptance Test A6: No Fourth Quarter From a Transition Period
//!
//! Runs `core/period/q4-from-fy@1` on the 10-K of Deckers for fiscal year 2013 and its 10-QT, after
//! the rules of one filing and `core/dedup/newest-filing@1`. Deckers moved its fiscal year end from
//! 31 December to 31 March. The 10-QT reports the transition period from 1 January to 31 March 2014.
//! Deckers filed no 10-KT for this change.
//!
//! The rule derives no fourth quarter. The 10-K reports no nine-month value, and the transition
//! period is one quarter, not a fiscal year.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use pretty_assertions::assert_eq;
use xbrl::core::period_identity::derive_q4;
use xbrl::core::standardized::StandardizedFact;
use xbrl::tests::fixtures::data::resolved::resolve_fixture_filings;

#[test]
fn should_derive_no_fourth_quarter_when_deckers_reports_its_transition_period_in_a_10_qt() {
    let series =
        resolve_fixture_filings(&["0001445305-14-000820", "0000910521-14-000013"]).series();

    let expected_result: Vec<StandardizedFact> = Vec::new();

    let result = derive_q4(&series);

    assert_eq!(result, expected_result);
}
