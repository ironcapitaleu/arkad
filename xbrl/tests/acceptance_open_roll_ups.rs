//! # Acceptance Test A2: Open Roll-Ups by Cause
//!
//! Lists the open roll-ups of the fixture 10-Ks that the spike named for a cause: General Mills,
//! APA, and KKR, and the bank, REIT, utility, and energy filers. A roll-up is open if no rule gives
//! a value for the fiscal year of the 10-K. The rules run as in A1 (`acceptance_roll_ups.rs`).
//!
//! - General Mills reports no liabilities and equity total for fiscal year 2013. The closed group
//!   `liabilities-and-equity` derives it with the redeemable interest of 967.5 million USD, which
//!   the part rule of `TemporaryEquity` resolves.
//! - APA reports revenue with `ExplorationAndProductionRevenue` in 2013 and with extension concepts
//!   in 2025. KKR reports no revenue total in 2013.
//! - The 2013 rows of `JPMorgan Chase`, Simon Property, and `ExxonMobil` carry `Revenues`, and the alias
//!   resolves it. Evergy reports `ElectricUtilityRevenue` in 2013, and `Revenues` in 2025. Regions
//!   Financial reports interest income and noninterest income, and no revenue total.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;
use xbrl::core::closed_group::derive_from_closed_groups;
use xbrl::core::confidence::Confidence;
use xbrl::core::elements::BalanceSheetConcept;
use xbrl::core::period_identity::derive_q4;
use xbrl::tests::fixtures::data::resolved::{
    SPIKE_ROLL_UPS, derived_values, fiscal_year_value, resolve_fixture_filings,
};
use xbrl::tests::fixtures::sample_fact::date;

/// Runs the rules of A1 on the fixture 10-K `accession`, and returns the name of each roll-up of
/// [`SPIKE_ROLL_UPS`] with no value for its fiscal year.
fn open_roll_ups(accession: &str) -> Vec<&'static str> {
    let resolved = resolve_fixture_filings(&[accession]);
    let year_end = resolved.filings[0]
        .period_of_report
        .expect("Given the sub row of a 10-K, the period of report should always exist");
    let series = resolved.series();
    let derived = derived_values(&series);
    SPIKE_ROLL_UPS
        .iter()
        .filter(|concept| {
            fiscal_year_value(series.facts.iter().chain(&derived), **concept, year_end).is_none()
        })
        .map(|concept| concept.name())
        .collect()
}

/// Writes one test per 10-K. Each lists the open roll-ups of the 10-K.
macro_rules! open_roll_up_tests {
    ($($name:ident: $accession:literal => [$($concept:literal),* $(,)?],)+) => {
        $(
            #[test]
            fn $name() {
                let accession = $accession;

                let expected_result: Vec<&str> = vec![$($concept),*];

                let result = open_roll_ups(accession);

                assert_eq!(result, expected_result);
            }
        )+
    };
}

open_roll_up_tests! {
    should_list_the_open_roll_ups_for_general_mills_10_k_2013_fy: "0001193125-13-282371" => [
        "NetChangeInCash",
    ],
    should_list_the_open_roll_ups_for_apa_10_k_2013_fy: "0001193125-14-077668" => [
        "Revenue",
        "NetChangeInCash",
    ],
    should_list_the_open_roll_ups_for_apa_10_k_2025_fy: "0001841666-26-000015" => [
        "Revenue",
    ],
    should_list_the_open_roll_ups_for_kkr_10_k_2013_fy: "0001047469-14-001189" => [
        "Revenue",
        "OperatingCashFlow",
        "InvestingCashFlow",
        "FinancingCashFlow",
        "NetChangeInCash",
    ],
    should_list_the_open_roll_ups_for_jpmorgan_chase_10_k_2013_fy: "0000019617-14-000289" => [
        "NetChangeInCash",
    ],
    should_list_the_open_roll_ups_for_jpmorgan_chase_10_k_2025_fy: "0001628280-26-008131" => [],
    should_list_the_open_roll_ups_for_simon_property_10_k_2013_fy: "0001047469-14-001416" => [
        "NetChangeInCash",
    ],
    should_list_the_open_roll_ups_for_simon_property_10_k_2025_fy: "0001104659-26-019419" => [],
    should_list_the_open_roll_ups_for_evergy_10_k_2013_fy: "0001143068-14-000021" => [
        "Revenue",
        "NetChangeInCash",
    ],
    should_list_the_open_roll_ups_for_evergy_10_k_2025_fy: "0001711269-26-000017" => [],
    should_list_the_open_roll_ups_for_exxonmobil_10_k_2013_fy: "0000034088-14-000012" => [
        "OperatingCashFlow",
        "InvestingCashFlow",
        "FinancingCashFlow",
        "NetChangeInCash",
    ],
    should_list_the_open_roll_ups_for_exxonmobil_10_k_2025_fy: "0000034088-26-000045" => [],
    should_list_the_open_roll_ups_for_regions_financial_10_k_2013_fy: "0001281761-14-000009" => [
        "Revenue",
        "NetChangeInCash",
    ],
    should_list_the_open_roll_ups_for_regions_financial_10_k_2025_fy: "0001281761-26-000019" => [
        "Revenue",
    ],
}

#[test]
fn should_derive_liabilities_and_equity_from_its_closed_group_for_general_mills_10_k_2013_fy() {
    let resolved = resolve_fixture_filings(&["0001193125-13-282371"]);
    let series = resolved.series();
    let fourth_quarters = derive_q4(&series);

    let expected_result = Some((
        Confidence::Derived,
        "core/roll-up/liabilities-and-equity@1".to_string(),
        BigDecimal::from(22_658_000_000_i64),
    ));

    let result = fiscal_year_value(
        &derive_from_closed_groups(&series, &fourth_quarters),
        BalanceSheetConcept::LiabilitiesAndEquity.element(),
        date(2013, 5, 26),
    )
    .map(|fact| (fact.confidence, fact.rule.to_string(), fact.value.clone()));

    assert_eq!(result, expected_result);
}
