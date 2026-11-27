//! # Acceptance Test A9: Segment Gaps
//!
//! Lists the axes of the candidates with dimensions of one 10-K, and the standard concepts that get
//! a candidate on a business-segment axis, after the rules of one filing with [`UsGaap`]. A business-segment axis is an axis of the kind
//! [`SegmentAxisKind::BusinessSegment`] in the segment axes of the plug-in.
//!
//! - Amazon gives no value on a business-segment axis for fiscal year 2014. Its values with
//!   dimensions lie on the geography, product, and consolidation items axes.
//! - Regions Financial reports no revenue total, so revenue has no segment value for fiscal year
//!   2021. Its Notes rows still put goodwill, income, and tax on `BusinessSegments`.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use std::collections::BTreeSet;

use pretty_assertions::assert_eq;
use xbrl::core::taxonomy::segment::{SegmentAxes, SegmentAxis, SegmentAxisKind};
use xbrl::tests::fixtures::data::resolved::resolve_fixture_filings;
use xbrl::us_gaap::UsGaap;

/// Returns the axes of the candidates with dimensions of the fixture 10-K `accession`, and the name
/// of each standard concept with a candidate on a business-segment axis of [`UsGaap`].
fn segment_axes_and_concepts(accession: &str) -> (BTreeSet<String>, BTreeSet<&'static str>) {
    let us_gaap = UsGaap::new();
    let business_segment_axes: Vec<&str> = us_gaap
        .segment_axes()
        .iter()
        .filter(|axis| axis.kind() == SegmentAxisKind::BusinessSegment)
        .map(SegmentAxis::name)
        .collect();
    let candidates = resolve_fixture_filings(&[accession]).candidates;
    let axes = candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .dimensions
                .iter()
                .map(|(axis, _)| axis.to_string())
        })
        .collect();
    let concepts = candidates
        .iter()
        .filter(|candidate| {
            candidate
                .dimensions
                .iter()
                .any(|(axis, _)| business_segment_axes.contains(&axis))
        })
        .map(|candidate| candidate.concept.name())
        .collect();
    (axes, concepts)
}

#[test]
fn should_give_no_business_segment_value_for_amazon_10_k_2014_fy() {
    let accession = "0001018724-15-000006";

    let expected_result = (
        BTreeSet::from([
            "ConsolidationItems".to_string(),
            "Geographical".to_string(),
            "ProductOrService".to_string(),
        ]),
        BTreeSet::new(),
    );

    let result = segment_axes_and_concepts(accession);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_no_business_segment_value_of_revenue_for_regions_financial_10_k_2021_fy() {
    let accession = "0001281761-22-000016";

    let expected_result = (
        BTreeSet::from(["BusinessSegments".to_string()]),
        BTreeSet::from(["Goodwill", "IncomeTaxExpense", "NetIncome", "PretaxIncome"]),
    );

    let result = segment_axes_and_concepts(accession);

    assert_eq!(result, expected_result);
}
