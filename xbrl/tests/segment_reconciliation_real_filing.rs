//! # Segment Reconciliation Real Filing Integration Tests
//!
//! Builds the segment reconciliation of one real filing, the 10-Q `0001628280-26-068246`, as
//! [`xbrl::adapter::edgar::notes::parse`] gives it.
//!
//! Each segment value and each consolidated revenue of a quarter is a conflict between a quarter
//! and a period of four months, so each candidate is blocked. The reconciliation then has no
//! unreconciled line.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use std::collections::BTreeSet;

use pretty_assertions::assert_eq;
use xbrl::core::alias::resolve_aliases;
use xbrl::core::dedup::more_precise;
use xbrl::core::segment::{ReconciliationLine, reconcile};
use xbrl::core::taxonomy::registry::{Plugin, Registry};
use xbrl::tests::fixtures::data::notes::parse_notes_fixture;
use xbrl::us_gaap::UsGaap;

#[test]
fn should_give_no_unreconciled_line_for_the_blocked_segments_of_the_fixture_filing() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let candidates =
        resolve_aliases(&more_precise(&parse_notes_fixture().facts), &registry).candidates;

    let expected_result = BTreeSet::from([false]);

    let result: BTreeSet<bool> = reconcile(&candidates, &registry)
        .iter()
        .map(|reconciliation| {
            reconciliation
                .lines
                .iter()
                .any(|line| matches!(line, ReconciliationLine::Unreconciled(_)))
        })
        .collect();

    assert_eq!(result, expected_result);
}
