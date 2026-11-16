//! # Segment Reconciliation Real Filing Integration Tests
//!
//! Builds the segment reconciliation of one real filing, the 10-Q `0001628280-26-068246`, as
//! [`xbrl::adapter::edgar::notes::parse`] gives it.
//!
//! The filer has one reportable segment. Each of the 10 reconciliations is a quarter. It holds a
//! reported consolidated value, one reported segment line, and an unreconciled line of zero. The
//! period of four months that shares the value key of each quarter has another exact start, so it
//! blocks no candidate.
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
/// and so on, or `"unreconciled"` and its value.
fn line_states(reconciliation: &Reconciliation) -> Vec<String> {
    reconciliation
        .lines
        .iter()
        .map(|line| match line {
            ReconciliationLine::Segment(candidate) if is_blocked(candidate) => {
                "blocked segment".to_string()
            }
            ReconciliationLine::Segment(_) => "reported segment".to_string(),
            ReconciliationLine::Elimination(_) => "elimination".to_string(),
            ReconciliationLine::Corporate(_) => "corporate".to_string(),
            ReconciliationLine::Unreconciled(value) => format!("unreconciled {value}"),
        })
        .collect()
}

/// Returns `true` if the value of `candidate` is blocked.
fn is_blocked(candidate: &Candidate) -> bool {
    candidate.value == CandidateValue::Blocked
}

#[test]
fn should_reconcile_the_reportable_segment_to_zero_for_the_quarters_of_the_fixture_filing() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let candidates =
        resolve_aliases(&more_precise(&parse_notes_fixture().facts), &registry).candidates;

    let expected_result = vec![
        (
            Some(false),
            vec!["reported segment".to_string(), "unreconciled 0".to_string()]
        );
        10
    ];

    let result: Vec<(Option<bool>, Vec<String>)> = reconcile(&candidates, &registry)
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
