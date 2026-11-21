//! # Acceptance Test A9: Segment Gaps
//!
//! Lists the standard concepts that get a value on a business-segment axis from one 10-K, after the
//! rules of one filing with [`UsGaap`]. A business-segment axis is an axis of the kind
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

/// Returns the name of each standard concept with a candidate of the fixture 10-K `accession` on a
/// business-segment axis of [`UsGaap`].
fn concepts_with_business_segments(accession: &str) -> BTreeSet<&'static str> {
    let us_gaap = UsGaap::new();
    let business_segment_axes: Vec<&str> = us_gaap
        .segment_axes()
        .iter()
        .filter(|axis| axis.kind() == SegmentAxisKind::BusinessSegment)
        .map(SegmentAxis::name)
        .collect();
    resolve_fixture_filings(&[accession])
        .candidates
        .iter()
        .filter(|candidate| {
            candidate
                .dimensions
                .iter()
                .any(|(axis, _)| business_segment_axes.contains(&axis))
        })
        .map(|candidate| candidate.concept.name())
        .collect()
}

#[test]
fn should_give_no_business_segment_value_for_amazon_10_k_2014_fy() {
    let accession = "0001018724-15-000006";

    let expected_result: BTreeSet<&str> = BTreeSet::new();

    let result = concepts_with_business_segments(accession);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_no_business_segment_value_of_revenue_for_regions_financial_10_k_2021_fy() {
    let accession = "0001281761-22-000016";

    let expected_result =
        BTreeSet::from(["Goodwill", "IncomeTaxExpense", "NetIncome", "PretaxIncome"]);

    let result = concepts_with_business_segments(accession);

    assert_eq!(result, expected_result);
}
