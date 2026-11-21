//! # Acceptance Test A8: Segment Sums
//!
//! Builds the segment reconciliation of revenue for fiscal year 2025 from the Coca-Cola 10-K, and
//! lists the revenue values on the geography axis. The rules of one filing run with [`UsGaap`].
//!
//! - The five operating segments sum to 48,806 million USD. With the elimination of −1,009
//!   million and the corporate line of 144 million, they give the consolidated revenue of 47,941
//!   million. The filer also tags 48,806 million with the member `MaterialReconcilingItems`. The
//!   US GAAP plug-in gives this member the corporate role, so the reconciliation counts the segment
//!   total twice, and its unreconciled line is −48,806 million.
//! - On the geography axis, the United States gives 19,127 million and the rest of the world
//!   28,814 million. They sum to the consolidated revenue.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;
use xbrl::core::candidate::{Candidate, CandidateValue};
use xbrl::core::segment::{Reconciliation, ReconciliationLine, reconcile};
use xbrl::core::taxonomy::registry::{Plugin, Registry};
use xbrl::tests::fixtures::data::resolved::resolve_fixture_filings;
use xbrl::tests::fixtures::sample_fact::date;
use xbrl::us_gaap::UsGaap;

/// The accession of the Coca-Cola 10-K for fiscal year 2025.
const COCA_COLA_10_K_2025: &str = "0001628280-26-010047";

/// The name of the revenue concept.
const REVENUE: &str = "Revenue";

/// The axis of the geography values.
const GEOGRAPHY_AXIS: &str = "Geographical";

/// The length of a fiscal year, in quarters.
const YEAR_LENGTH: u16 = 4;

/// Returns the value of `candidate`, or `None` if it is blocked.
fn value_of(candidate: &Candidate) -> Option<BigDecimal> {
    match &candidate.value {
        CandidateValue::Reported { value, .. } => Some(value.clone()),
        CandidateValue::Blocked => None,
    }
}

/// Returns the members of `candidate`, joined by `+`.
fn members_of(candidate: &Candidate) -> String {
    candidate
        .dimensions
        .iter()
        .map(|(_, member)| member)
        .collect::<Vec<_>>()
        .join("+")
}

/// Returns the kind, the members, and the value of each line of `reconciliation`.
fn lines_of(reconciliation: &Reconciliation) -> Vec<(&'static str, String, Option<BigDecimal>)> {
    reconciliation
        .lines
        .iter()
        .map(|line| match line {
            ReconciliationLine::Segment(candidate) => {
                ("segment", members_of(candidate), value_of(candidate))
            }
            ReconciliationLine::Elimination(candidate) => {
                ("elimination", members_of(candidate), value_of(candidate))
            }
            ReconciliationLine::Corporate(candidate) => {
                ("corporate", members_of(candidate), value_of(candidate))
            }
            ReconciliationLine::Unreconciled(value) => {
                ("unreconciled", String::new(), Some(value.clone()))
            }
        })
        .collect()
}

/// Returns the line of `kind` with `members` and `value` in millions of USD.
fn line(
    kind: &'static str,
    members: &str,
    millions: i64,
) -> (&'static str, String, Option<BigDecimal>) {
    (
        kind,
        members.to_string(),
        Some(BigDecimal::from(millions * 1_000_000)),
    )
}

#[test]
fn should_count_the_segment_total_twice_when_coca_cola_tags_it_as_a_material_reconciling_item() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the creation should always succeed");
    let candidates = resolve_fixture_filings(&[COCA_COLA_10_K_2025]).candidates;

    let expected_result = vec![vec![
        line("segment", "A.Pacific+OperatingSegments", 5_638),
        line("segment", "BottlingInvestments+OperatingSegments", 5_735),
        line(
            "segment",
            "EuropeMiddleEastAfrica+OperatingSegments",
            11_513,
        ),
        line("segment", "LatinAmericaSegment+OperatingSegments", 6_334),
        line("segment", "NorthAmericaSegment+OperatingSegments", 19_586),
        line("elimination", "ConsolidationEliminations", -1_009),
        line("corporate", "CorporateNonSegment", 144),
        line("corporate", "MaterialReconcilingItems", 48_806),
        line("unreconciled", "", -48_806),
    ]];

    let result: Vec<_> = reconcile(&candidates, &registry)
        .iter()
        .filter(|reconciliation| {
            reconciliation.concept.name() == REVENUE
                && reconciliation.period_end == date(2025, 12, 31)
                && reconciliation.quarters == YEAR_LENGTH
        })
        .map(lines_of)
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_split_revenue_into_the_united_states_and_the_rest_of_the_world_for_coca_cola_10_k_2025_fy()
 {
    let candidates = resolve_fixture_filings(&[COCA_COLA_10_K_2025]).candidates;

    let expected_result = vec![
        (
            "NonUs".to_string(),
            Some(BigDecimal::from(28_814_000_000_i64)),
        ),
        ("US".to_string(), Some(BigDecimal::from(19_127_000_000_i64))),
    ];

    let result: Vec<(String, Option<BigDecimal>)> = candidates
        .iter()
        .filter(|candidate| {
            candidate.concept.name() == REVENUE
                && candidate.period.end() == date(2025, 12, 31)
                && candidate.period.quarters() == YEAR_LENGTH
                && candidate
                    .dimensions
                    .iter()
                    .map(|(axis, _)| axis)
                    .eq([GEOGRAPHY_AXIS])
        })
        .map(|candidate| (members_of(candidate), value_of(candidate)))
        .collect();

    assert_eq!(result, expected_result);
}
