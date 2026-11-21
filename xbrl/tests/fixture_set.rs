//! # Fixture Set Integration Tests
//!
//! Runs [`xbrl::adapter::edgar::notes::parse`] on each compressed fixture filing of the manifest:
//! 105 filings of 33 companies. `manifest.tsv` names the form and the fiscal year of each.
//! Each smoke test checks that the parser maps every row of one filing, so it gives no
//! [`Finding::UnmappedRow`]. Three more tests check the manifest: each fixture file matches its
//! SHA-256, each file on disk has a row, and each Notes filing has a smoke test.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`, and `manifest.tsv` lists them.

use std::collections::BTreeSet;
use std::fmt::Write;

use pretty_assertions::assert_eq;
use sha2::{Digest, Sha256};
use xbrl::core::finding::Finding;
use xbrl::tests::fixtures::data::manifest::manifest;
use xbrl::tests::fixtures::data::notes::parse_notes_fixture_filing;
use xbrl::tests::fixtures::data::{
    GZIP_EXTENSION, content_of, fixture_file, fixture_files_on_disk,
};

/// The prefix of the path of a Notes fixture file in the manifest.
const NOTES_PREFIX: &str = "notes/";

/// Returns the [`Finding::UnmappedRow`] findings of the fixture filing `accession`.
fn unmapped_rows(accession: &str) -> Vec<Finding> {
    parse_notes_fixture_filing(accession)
        .findings
        .into_iter()
        .filter(|finding| matches!(finding, Finding::UnmappedRow { .. }))
        .collect()
}

/// Returns the SHA-256 of `bytes` as 64 lowercase hex digits.
fn hex_digest(bytes: &[u8]) -> String {
    let mut hex = String::new();
    for byte in Sha256::digest(bytes) {
        write!(hex, "{byte:02x}").expect("Given a String, a write should never fail");
    }
    hex
}

/// Writes one smoke test per fixture filing, and [`SMOKE_TESTED`], the list of their accessions.
macro_rules! smoke_tests {
    ($($name:ident: $accession:literal,)+) => {
        /// The accession of each fixture filing that a smoke test parses.
        const SMOKE_TESTED: &[&str] = &[$($accession),+];

        $(
            #[test]
            fn $name() {
                let accession = $accession;

                let expected_result: Vec<Finding> = Vec::new();

                let result = unmapped_rows(accession);

                assert_eq!(result, expected_result);
            }
        )+
    };
}

smoke_tests! {
    should_map_every_row_for_3m_10_k_2013_fy: "0001104659-14-009773",
    should_map_every_row_for_3m_10_k_2025_fy: "0000066740-26-000014",
    should_map_every_row_for_abbvie_10_k_2013_fy: "0001047469-14-001154",
    should_map_every_row_for_abbvie_10_k_2025_fy: "0001551152-26-000008",
    should_map_every_row_for_aes_10_k_2013_fy: "0000874761-14-000011",
    should_map_every_row_for_aes_10_k_2025_fy: "0000874761-26-000063",
    should_map_every_row_for_air_products_10_k_2013_fy: "0001193125-13-453925",
    should_map_every_row_for_air_products_10_k_2025_fy: "0000002969-25-000055",
    should_map_every_row_for_alphabet_10_k_2013_fy: "0001288776-14-000020",
    should_map_every_row_for_alphabet_10_k_2025_fy: "0001652044-26-000018",
    should_map_every_row_for_amazon_10_k_2014_fy: "0001018724-15-000006",
    should_map_every_row_for_amazon_10_k_2025_fy: "0001018724-26-000004",
    should_map_every_row_for_amd_10_k_2013_fy: "0001193125-14-057240",
    should_map_every_row_for_amd_10_k_2025_fy: "0000002488-26-000018",
    should_map_every_row_for_amd_10_q_2025_q1: "0000002488-25-000047",
    should_map_every_row_for_amd_10_q_2025_q2: "0000002488-25-000108",
    should_map_every_row_for_amd_10_q_2025_q3: "0000002488-25-000166",
    should_map_every_row_for_amd_10_q_2026_q2: "0000002488-26-000123",
    should_map_every_row_for_ameren_10_k_2013_fy: "0001002910-14-000054",
    should_map_every_row_for_ameren_10_k_2025_fy: "0001002910-26-000009",
    should_map_every_row_for_apa_10_k_2013_fy: "0001193125-14-077668",
    should_map_every_row_for_apa_10_k_2025_fy: "0001841666-26-000015",
    should_map_every_row_for_apple_10_k_2013_fy: "0001193125-13-416534",
    should_map_every_row_for_apple_10_k_2025_fy: "0000320193-25-000079",
    should_map_every_row_for_apple_10_q_2025_q1: "0000320193-25-000008",
    should_map_every_row_for_apple_10_q_2025_q2: "0000320193-25-000057",
    should_map_every_row_for_apple_10_q_2025_q3: "0000320193-25-000073",
    should_map_every_row_for_apple_10_q_2026_q3: "0000320193-26-000020",
    should_map_every_row_for_baker_hughes_10_k_2013_fy: "0000808362-14-000007",
    should_map_every_row_for_baker_hughes_10_k_2025_fy: "0001701605-26-000007",
    should_map_every_row_for_coca_cola_10_k_2013_fy: "0000021344-14-000008",
    should_map_every_row_for_coca_cola_10_k_2025_fy: "0001628280-26-010047",
    should_map_every_row_for_coca_cola_10_q_2025_q1: "0000021344-25-000029",
    should_map_every_row_for_coca_cola_10_q_2025_q2: "0000021344-25-000061",
    should_map_every_row_for_coca_cola_10_q_2025_q3: "0001628280-25-046054",
    should_map_every_row_for_costco_10_k_2013_fy: "0001445305-13-002422",
    should_map_every_row_for_costco_10_k_2025_fy: "0000909832-25-000101",
    should_map_every_row_for_crown_castle_10_k_2013_fy: "0001051470-14-000008",
    should_map_every_row_for_crown_castle_10_k_2025_fy: "0001051470-26-000016",
    should_map_every_row_for_deckers_10_k_2013_fy: "0001445305-14-000820",
    should_map_every_row_for_deckers_10_qt_2013_q4: "0000910521-14-000013",
    should_map_every_row_for_deckers_10_k_2025_fy: "0000910521-25-000017",
    should_map_every_row_for_deckers_10_q_2025_q1: "0000910521-24-000042",
    should_map_every_row_for_deckers_10_q_2025_q2: "0000910521-24-000059",
    should_map_every_row_for_deckers_10_q_2025_q3: "0000910521-25-000005",
    should_map_every_row_for_digital_realty_10_k_2013_fy: "0001193125-14-078397",
    should_map_every_row_for_digital_realty_10_k_2025_fy: "0001104659-26-015365",
    should_map_every_row_for_disney_10_k_2013_fy: "0001001039-13-000164",
    should_map_every_row_for_disney_10_k_2025_fy: "0001744489-25-000155",
    should_map_every_row_for_disney_10_q_2025_q1: "0001744489-25-000067",
    should_map_every_row_for_disney_10_q_2025_q2: "0001744489-25-000098",
    should_map_every_row_for_disney_10_q_2025_q3: "0001744489-25-000136",
    should_map_every_row_for_disney_10_q_2026_q3: "0001744489-26-000057",
    should_map_every_row_for_dow_10_k_2013_fy: "0000029915-14-000011",
    should_map_every_row_for_dow_10_k_2025_fy: "0001751788-26-000018",
    should_map_every_row_for_emerson_10_k_2013_fy: "0000032604-13-000044",
    should_map_every_row_for_emerson_10_k_2025_fy: "0000032604-25-000087",
    should_map_every_row_for_evergy_10_k_2013_fy: "0001143068-14-000021",
    should_map_every_row_for_evergy_10_k_2025_fy: "0001711269-26-000017",
    should_map_every_row_for_evergy_10_q_2025_q1: "0001711269-25-000010",
    should_map_every_row_for_evergy_10_q_2025_q2: "0001711269-25-000022",
    should_map_every_row_for_evergy_10_q_2025_q3: "0001711269-25-000041",
    should_map_every_row_for_exxonmobil_10_k_2013_fy: "0000034088-14-000012",
    should_map_every_row_for_exxonmobil_10_k_2025_fy: "0000034088-26-000045",
    should_map_every_row_for_exxonmobil_10_q_2025_q1: "0000034088-25-000024",
    should_map_every_row_for_exxonmobil_10_q_2025_q2: "0000034088-25-000042",
    should_map_every_row_for_exxonmobil_10_q_2025_q3: "0000034088-25-000061",
    should_map_every_row_for_general_mills_10_k_2013_fy: "0001193125-13-282371",
    should_map_every_row_for_general_mills_10_k_2025_fy: "0001193125-25-147079",
    should_map_every_row_for_intuitive_surgical_10_k_2013_fy: "0001035267-14-000009",
    should_map_every_row_for_intuitive_surgical_10_k_2025_fy: "0001035267-26-000010",
    should_map_every_row_for_jpmorgan_chase_10_k_2013_fy: "0000019617-14-000289",
    should_map_every_row_for_jpmorgan_chase_10_k_2025_fy: "0001628280-26-008131",
    should_map_every_row_for_jpmorgan_chase_10_q_2025_q1: "0000019617-25-000421",
    should_map_every_row_for_jpmorgan_chase_10_q_2025_q2: "0000019617-25-000615",
    should_map_every_row_for_jpmorgan_chase_10_q_2025_q3: "0001628280-25-048859",
    should_map_every_row_for_kkr_10_k_2013_fy: "0001047469-14-001189",
    should_map_every_row_for_kkr_10_k_2025_fy: "0001404912-26-000007",
    should_map_every_row_for_medtronic_10_k_2013_fy: "0000064670-13-000006",
    should_map_every_row_for_medtronic_10_k_2025_fy: "0001613103-25-000091",
    should_map_every_row_for_procter_and_gamble_10_k_2013_fy: "0000080424-13-000063",
    should_map_every_row_for_procter_and_gamble_10_k_2025_fy: "0000080424-25-000076",
    should_map_every_row_for_procter_and_gamble_10_k_2026_fy: "0000080424-26-000103",
    should_map_every_row_for_prudential_10_k_2013_fy: "0001193125-14-073074",
    should_map_every_row_for_prudential_10_k_2025_fy: "0001137774-26-000048",
    should_map_every_row_for_regions_financial_10_k_2013_fy: "0001281761-14-000009",
    should_map_every_row_for_regions_financial_10_k_2021_fy: "0001281761-22-000016",
    should_map_every_row_for_regions_financial_10_k_2025_fy: "0001281761-26-000019",
    should_map_every_row_for_simon_property_10_k_2013_fy: "0001047469-14-001416",
    should_map_every_row_for_simon_property_10_k_2025_fy: "0001104659-26-019419",
    should_map_every_row_for_simon_property_10_q_2025_q1: "0001558370-25-007380",
    should_map_every_row_for_simon_property_10_q_2025_q2: "0001558370-25-010639",
    should_map_every_row_for_simon_property_10_q_2025_q3: "0001104659-25-107395",
    should_map_every_row_for_supermicro_10_k_2013_fy: "0001445305-13-002262",
    should_map_every_row_for_supermicro_10_k_2017_fy: "0001375365-19-000039",
    should_map_every_row_for_supermicro_10_k_2019_fy: "0001375365-19-000079",
    should_map_every_row_for_supermicro_10_k_2025_fy: "0001375365-25-000027",
    should_map_every_row_for_unitedhealth_10_k_2013_fy: "0000731766-14-000008",
    should_map_every_row_for_unitedhealth_10_k_2025_fy: "0000731766-26-000062",
    should_map_every_row_for_walmart_10_k_2013_fy: "0000104169-14-000019",
    should_map_every_row_for_walmart_10_k_2025_fy: "0000104169-25-000021",
    should_map_every_row_for_walmart_10_q_2025_q1: "0000104169-24-000105",
    should_map_every_row_for_walmart_10_q_2025_q2: "0000104169-24-000141",
    should_map_every_row_for_walmart_10_q_2025_q3: "0000104169-24-000178",
    should_map_every_row_for_walmart_10_q_2027_q2: "0000104169-26-000154",
}

#[test]
fn should_match_the_sha256_of_the_manifest_for_every_fixture_file() {
    let entries = manifest();

    let expected_result: Vec<String> = Vec::new();

    let result: Vec<String> = entries
        .iter()
        .filter(|entry| {
            let stored = fixture_file(&entry.file);
            hex_digest(&stored) != entry.sha256
                || hex_digest(&content_of(&entry.file, stored)) != entry.sha256_uncompressed
        })
        .map(|entry| entry.file.clone())
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_have_a_smoke_test_for_every_compressed_notes_filing_of_the_manifest() {
    let entries = manifest();

    let expected_result: BTreeSet<&str> = SMOKE_TESTED.iter().copied().collect();

    let result: BTreeSet<&str> = entries
        .iter()
        .filter(|entry| {
            entry.file.starts_with(NOTES_PREFIX) && entry.file.ends_with(GZIP_EXTENSION)
        })
        .map(|entry| entry.accession.as_str())
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_list_every_fixture_file_on_disk_in_the_manifest() {
    let entries = manifest();

    let expected_result: BTreeSet<String> =
        entries.iter().map(|entry| entry.file.clone()).collect();

    let result: BTreeSet<String> = fixture_files_on_disk().into_iter().collect();

    assert_eq!(result, expected_result);
}
