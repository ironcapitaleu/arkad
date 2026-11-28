//! # Missing Required Concepts of the Fixture 10-Ks
//!
//! Runs [`standardize_series`] with the required concepts of the US GAAP plug-in on each fixture
//! 10-K of A2 (`acceptance_open_roll_ups.rs`), and lists its `MissingRequired` findings. Each
//! finding names a roll-up that A2 lists as open. No filing tags one of these roll-ups for any
//! period, so each reason is `Untagged`.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use pretty_assertions::assert_eq;
use xbrl::core::finding::{Finding, MissingReason};
use xbrl::core::series::standardize_series;
use xbrl::tests::fixtures::data::resolved::resolve_fixture_filings;
use xbrl::us_gaap::UsGaap;

/// Runs [`standardize_series`] on the fixture 10-K `accession`, and returns the concept and the
/// reason of each `MissingRequired` finding.
fn missing_required(accession: &str) -> Vec<(&'static str, Option<MissingReason>)> {
    let resolved = resolve_fixture_filings(&[accession]);
    let required = UsGaap::new().required_concepts();
    standardize_series(
        &resolved.candidates,
        &resolved.filings,
        Vec::new(),
        &required,
    )
    .findings
    .iter()
    .filter_map(|finding| match finding {
        Finding::MissingRequired {
            concept, reason, ..
        } => Some((concept.name(), *reason)),
        _ => None,
    })
    .collect()
}

/// Writes one test per 10-K. Each lists the missing required concepts of the 10-K.
macro_rules! missing_required_tests {
    ($($name:ident: $accession:literal => [$($concept:literal),* $(,)?],)+) => {
        $(
            #[test]
            fn $name() {
                let accession = $accession;

                let expected_result: Vec<(&str, Option<MissingReason>)> =
                    vec![$(($concept, Some(MissingReason::Untagged))),*];

                let result = missing_required(accession);

                assert_eq!(result, expected_result);
            }
        )+
    };
}

missing_required_tests! {
    should_list_the_missing_required_concepts_for_general_mills_10_k_2013_fy: "0001193125-13-282371" => [
        "NetChangeInCash",
    ],
    should_list_the_missing_required_concepts_for_apa_10_k_2013_fy: "0001193125-14-077668" => [
        "Revenue",
        "NetChangeInCash",
    ],
    should_list_the_missing_required_concepts_for_apa_10_k_2025_fy: "0001841666-26-000015" => [
        "Revenue",
    ],
    should_list_the_missing_required_concepts_for_kkr_10_k_2013_fy: "0001047469-14-001189" => [
        "Revenue",
        "NetChangeInCash",
    ],
    should_list_the_missing_required_concepts_for_jpmorgan_chase_10_k_2013_fy: "0000019617-14-000289" => [
        "NetChangeInCash",
    ],
    should_list_the_missing_required_concepts_for_jpmorgan_chase_10_k_2025_fy: "0001628280-26-008131" => [],
    should_list_the_missing_required_concepts_for_simon_property_10_k_2013_fy: "0001047469-14-001416" => [
        "NetChangeInCash",
    ],
    should_list_the_missing_required_concepts_for_simon_property_10_k_2025_fy: "0001104659-26-019419" => [],
    should_list_the_missing_required_concepts_for_evergy_10_k_2013_fy: "0001143068-14-000021" => [
        "Revenue",
        "NetChangeInCash",
    ],
    should_list_the_missing_required_concepts_for_evergy_10_k_2025_fy: "0001711269-26-000017" => [],
    should_list_the_missing_required_concepts_for_exxonmobil_10_k_2013_fy: "0000034088-14-000012" => [
        "NetChangeInCash",
    ],
    should_list_the_missing_required_concepts_for_exxonmobil_10_k_2025_fy: "0000034088-26-000045" => [],
    should_list_the_missing_required_concepts_for_regions_financial_10_k_2013_fy: "0001281761-14-000009" => [
        "Revenue",
        "NetChangeInCash",
    ],
    should_list_the_missing_required_concepts_for_regions_financial_10_k_2025_fy: "0001281761-26-000019" => [
        "Revenue",
    ],
}
