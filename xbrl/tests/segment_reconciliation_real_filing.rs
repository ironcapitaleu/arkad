//! # Segment Reconciliation Real Filing Integration Tests
//!
//! Builds the segment reconciliation of one real filing, the 10-Q `0001628280-26-068246`, as
//! [`xbrl::adapter::edgar::notes::parse`] gives it.
//!
//! Each segment value and each consolidated revenue of a quarter is a conflict between a quarter
//! and a period of four months, so each candidate is blocked. Each of the 10 reconciliations holds a
//! blocked consolidated value and one blocked segment line, and no unreconciled line.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use pretty_assertions::assert_eq;
use xbrl::core::alias::resolve_aliases;
use xbrl::core::candidate::{Candidate, CandidateValue};
use xbrl::core::dedup::more_precise;
use xbrl::core::segment::{Reconciliation, ReconciliationLine, reconcile};
use xbrl::core::taxonomy::registry::{Plugin, Registry};
use xbrl::tests::fixtures::data::notes::parse_notes_fixture;
use xbrl::us_gaap::UsGaap;

/// Returns the state of each line of `reconciliation`: `"reported segment"`, `"blocked segment"`,
/// and so on, or `"unreconciled"`.
fn line_states(reconciliation: &Reconciliation) -> Vec<&'static str> {
    reconciliation
        .lines
        .iter()
        .map(|line| match line {
            ReconciliationLine::Segment(candidate) if is_blocked(candidate) => "blocked segment",
            ReconciliationLine::Segment(_) => "reported segment",
            ReconciliationLine::Elimination(_) => "elimination",
            ReconciliationLine::Corporate(_) => "corporate",
            ReconciliationLine::Unreconciled(_) => "unreconciled",
        })
        .collect()
}

/// Returns `true` if the value of `candidate` is blocked.
fn is_blocked(candidate: &Candidate) -> bool {
    candidate.value == CandidateValue::Blocked
}

#[test]
fn should_give_no_unreconciled_line_for_the_blocked_segments_of_the_fixture_filing() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let candidates =
        resolve_aliases(&more_precise(&parse_notes_fixture().facts), &registry).candidates;

    let expected_result = vec![(Some(true), vec!["blocked segment"]); 10];

    let result: Vec<(Option<bool>, Vec<&str>)> = reconcile(&candidates, &registry)
        .iter()
        .map(|reconciliation| {
            (
                reconciliation.consolidated.as_ref().map(is_blocked),
                line_states(reconciliation),
            )
        })
        .collect();

    assert_eq!(result, expected_result);
}
