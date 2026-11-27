//! # Acceptance Test A5: Fourth Quarter From the Fiscal Year
//!
//! Runs `core/period/q4-from-fy@1` on the 10-K and the three 10-Qs of fiscal year 2025 of each of
//! the 10 companies with 10-Qs in the fixture set, after the rules of one filing and
//! `core/dedup/newest-filing@1`. Each test lists the derived fourth quarter of fiscal year 2025 for
//! revenue, net income, and operating cash flow. Each value equals the fiscal year less the nine
//! months of the third 10-Q in the rows of the filings.
//!
//! The rule derives no fourth quarter that the series already holds or that has no nine-month
//! value:
//!
//! - `JPMorgan Chase` reports no nine-month revenue total in its 10-Q.
//! - Simon Property reports the fourth quarter of revenue and net income in its 10-K.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use xbrl::core::period_identity::derive_q4;
use xbrl::core::standardized::StandardizedFact;
use xbrl::tests::fixtures::data::resolved::resolve_fixture_filings;
use xbrl::tests::fixtures::sample_fact::date;

/// The names of the concepts the test lists, in the order of the derived facts.
const CONCEPTS: [&str; 3] = ["Revenue", "NetIncome", "OperatingCashFlow"];

/// The confidence and the rule of every derived fourth quarter.
const Q4_RULE: &str = "Derived core/period/q4-from-fy@1";

/// The concept, the period, the value, and the confidence with the rule of a derived fact.
type Summary = (&'static str, String, BigDecimal, String);

/// Returns the [`Summary`] of each fact of `derived` that ends at `year_end` and has a concept of
/// [`CONCEPTS`].
fn fourth_quarters(derived: &[StandardizedFact], year_end: NaiveDate) -> Vec<Summary> {
    derived
        .iter()
        .filter(|fact| fact.period.end() == year_end && CONCEPTS.contains(&fact.concept.name()))
        .map(|fact| {
            (
                fact.concept.name(),
                fact.period.to_string(),
                fact.value.clone(),
                format!("{} {}", fact.confidence, fact.rule),
            )
        })
        .collect()
}

/// Writes one test per company. Each lists the fourth quarters that end at the end of fiscal year
/// 2025.
macro_rules! fourth_quarter_tests {
    ($(
        $name:ident: [$($accession:literal),+], $year_end:expr => [
            $(($concept:literal, $period:literal, $value:literal),)*
        ],
    )+) => {
        $(
            #[test]
            fn $name() {
                let series = resolve_fixture_filings(&[$($accession),+]).series();
                let year_end: NaiveDate = $year_end;

                let expected_result: Vec<Summary> = vec![
                    $(($concept, $period.to_string(), BigDecimal::from($value), Q4_RULE.to_string()),)*
                ];

                let result = fourth_quarters(&derive_q4(&series), year_end);

                assert_eq!(result, expected_result);
            }
        )+
    };
}

fourth_quarter_tests! {
    should_derive_the_fourth_quarter_for_amd_2025:
        ["0000002488-26-000018", "0000002488-25-000047", "0000002488-25-000108", "0000002488-25-000166"],
        date(2025, 12, 27) => [
        ("Revenue", "2025-09-28 to 2025-12-27", 10_270_000_000_i64),
        ("NetIncome", "2025-09-28 to 2025-12-27", 1_511_000_000_i64),
        ("OperatingCashFlow", "2025-09-28 to 2025-12-27", 2_600_000_000_i64),
    ],
    should_derive_the_fourth_quarter_for_apple_2025:
        ["0000320193-25-000079", "0000320193-25-000008", "0000320193-25-000057", "0000320193-25-000073"],
        date(2025, 9, 27) => [
        ("Revenue", "2025-06-29 to 2025-09-27", 102_466_000_000_i64),
        ("NetIncome", "2025-06-29 to 2025-09-27", 27_466_000_000_i64),
        ("OperatingCashFlow", "2025-06-29 to 2025-09-27", 29_728_000_000_i64),
    ],
    should_derive_the_fourth_quarter_for_coca_cola_2025:
        ["0001628280-26-010047", "0000021344-25-000029", "0000021344-25-000061", "0001628280-25-046054"],
        date(2025, 12, 31) => [
        ("Revenue", "2025-09-27 to 2025-12-31", 11_822_000_000_i64),
        ("NetIncome", "2025-09-27 to 2025-12-31", 2_316_000_000_i64),
        ("OperatingCashFlow", "2025-09-27 to 2025-12-31", 3_756_000_000_i64),
    ],
    should_derive_the_fourth_quarter_for_deckers_2025:
        ["0000910521-25-000017", "0000910521-24-000042", "0000910521-24-000059", "0000910521-25-000005"],
        date(2025, 3, 31) => [
        ("Revenue", "2025-01-01 to 2025-03-31", 1_021_780_000_i64),
        ("NetIncome", "2025-01-01 to 2025-03-31", 151_411_000_i64),
        ("OperatingCashFlow", "2025-01-01 to 2025-03-31", -72_992_000_i64),
    ],
    should_derive_the_fourth_quarter_for_disney_2025:
        ["0001744489-25-000155", "0001744489-25-000067", "0001744489-25-000098", "0001744489-25-000136"],
        date(2025, 9, 27) => [
        ("Revenue", "2025-06-29 to 2025-09-27", 22_464_000_000_i64),
        ("NetIncome", "2025-06-29 to 2025-09-27", 1_443_000_000_i64),
        ("OperatingCashFlow", "2025-06-29 to 2025-09-27", 4_474_000_000_i64),
    ],
    should_derive_the_fourth_quarter_for_evergy_2025:
        ["0001711269-26-000017", "0001711269-25-000010", "0001711269-25-000022", "0001711269-25-000041"],
        date(2025, 12, 31) => [
        ("Revenue", "2025-10-01 to 2025-12-31", 1_340_200_000_i64),
        ("NetIncome", "2025-10-01 to 2025-12-31", 87_400_000_i64),
        ("OperatingCashFlow", "2025-10-01 to 2025-12-31", 334_000_000_i64),
    ],
    should_derive_the_fourth_quarter_for_exxonmobil_2025:
        ["0000034088-26-000045", "0000034088-25-000024", "0000034088-25-000042", "0000034088-25-000061"],
        date(2025, 12, 31) => [
        ("Revenue", "2025-10-01 to 2025-12-31", 82_308_000_000_i64),
        ("NetIncome", "2025-10-01 to 2025-12-31", 6_609_000_000_i64),
        ("OperatingCashFlow", "2025-10-01 to 2025-12-31", 12_679_000_000_i64),
    ],
    should_derive_no_fourth_quarter_of_revenue_for_jpmorgan_chase_2025:
        ["0001628280-26-008131", "0000019617-25-000421", "0000019617-25-000615", "0001628280-25-048859"],
        date(2025, 12, 31) => [
        ("NetIncome", "2025-10-01 to 2025-12-31", 13_025_000_000_i64),
        ("OperatingCashFlow", "2025-10-01 to 2025-12-31", 119_724_000_000_i64),
    ],
    should_derive_only_operating_cash_flow_for_simon_property_2025:
        ["0001104659-26-019419", "0001558370-25-007380", "0001558370-25-010639", "0001104659-25-107395"],
        date(2025, 12, 31) => [
        ("OperatingCashFlow", "2025-10-01 to 2025-12-31", 1_203_741_000_i64),
    ],
    should_derive_the_fourth_quarter_for_walmart_2025:
        ["0000104169-25-000021", "0000104169-24-000105", "0000104169-24-000141", "0000104169-24-000178"],
        date(2025, 1, 31) => [
        ("Revenue", "2024-11-01 to 2025-01-31", 180_554_000_000_i64),
        ("NetIncome", "2024-11-01 to 2025-01-31", 5_425_000_000_i64),
        ("OperatingCashFlow", "2024-11-01 to 2025-01-31", 13_525_000_000_i64),
    ],
}
