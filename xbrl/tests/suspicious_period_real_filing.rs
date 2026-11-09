//! # Suspicious Period Real Filing Integration Tests
//!
//! Runs the period check on the revenue series of one real filing, the 10-Q `0001628280-26-068246`
//! of FEDEX FREIGHT HOLDING COMPANY, INC. The filer is a new registrant: its first filing dates
//! from 2025-08-29, and this 10-Q reports revenue for June 2025 as a comparative. The test shows
//! that the signals of the check leave such a comparative alone. The check has no signal for a
//! start before the first filing, so the test does not cover that signal.
//!
//! The fake plug-in holds the revenue alias list of the fixtures until the US GAAP plug-in has
//! its own lists.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use chrono::Datelike;
use pretty_assertions::assert_eq;
use xbrl::core::alias::resolve_aliases;
use xbrl::core::dedup::{Series, more_precise, newest_filing};
use xbrl::core::finding::Finding;
use xbrl::core::suspicious_period::suspicious_periods;
use xbrl::core::taxonomy::registry::Registry;
use xbrl::tests::fixtures::data::fixture_taxonomy_with_aliases;
use xbrl::tests::fixtures::data::notes::parse_notes_fixture;
use xbrl::tests::fixtures::sample_fact::revenue_aliases;

/// The year of the comparatives in the fixture filing.
const COMPARATIVE_YEAR: i32 = 2025;

/// Returns the revenue series of the Notes facts of the fixture filing.
fn revenue_series() -> Series {
    let facts = parse_notes_fixture().facts;
    let plugin = fixture_taxonomy_with_aliases(vec![revenue_aliases()]);
    let registry = Registry::new(vec![&plugin]).expect(
        "Given one plug-in with four distinct families, the creation should always succeed",
    );
    newest_filing(&resolve_aliases(&more_precise(&facts), &registry).candidates)
}

#[test]
fn should_flag_no_period_when_a_new_registrant_reports_a_comparative_of_the_year_before() {
    let series = revenue_series();
    let filings = vec![parse_notes_fixture().filing];
    series
        .facts
        .iter()
        .find(|fact| fact.period.end().year() == COMPARATIVE_YEAR)
        .expect(
            "Given the fixture filing, the revenue series should hold the June 2025 comparative",
        );

    let expected_result: Vec<Finding> = Vec::new();

    let result = suspicious_periods(&series, &filings);

    assert_eq!(result, expected_result);
}
