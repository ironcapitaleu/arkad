//! # Company Facts of Awkward Filers
//!
//! Company Facts JSON of four S&P 500 companies whose filings break the simple concept mapping.
//! Concept resolution tests use them to find the cases a clean filer hides.
//!
//! ## Source and Trim
//!
//! Each file comes from `https://data.sec.gov/api/xbrl/companyfacts/CIK##########.json`,
//! downloaded on 2026-11-08. The full files are 3 to 8 MB each, so each file keeps only the `dei`
//! facts and the `us-gaap` concepts that `concepts.txt` lists. Every kept fact is as the SEC sends
//! it, with its full history. To refresh a file, download it again and run:
//!
//! ```text
//! jq -c --rawfile c concepts.txt '($c|split("\n")|map(select(length>0))) as $keep
//!   | .facts = {dei: .facts.dei, "us-gaap": (.facts["us-gaap"]
//!   | with_entries(select(.key as $k | $keep|index($k))))}' raw.json > CIK##########.json
//! ```
//!
//! The Berkshire Hathaway file in the `sec` crate covers a conglomerate with an insurance arm.

/// Company Facts of `JPMorgan` Chase, a bank.
///
/// It reports no `OperatingIncomeLoss`. It tags both `Revenues` and `RevenuesNetOfInterestExpense`
/// for the same years, with different values.
pub const JPMORGAN_CHASE: &str = include_str!("CIK0000019617.json");

/// Company Facts of Progressive, a property and casualty insurer.
///
/// It reports no `OperatingIncomeLoss`. Its main revenue line is `PremiumsEarnedNet`.
pub const PROGRESSIVE: &str = include_str!("CIK0000080661.json");

/// Company Facts of Walmart, whose fiscal year ends on January 31.
///
/// Its fiscal year 2025 ends on 2025-01-31. It reports no `Liabilities` total, only
/// `LiabilitiesCurrent` and `LiabilitiesAndStockholdersEquity`.
pub const WALMART: &str = include_str!("CIK0000104169.json");

/// Company Facts of Simon Property Group, a REIT.
///
/// It reports no `NetIncomeLoss`, only `ProfitLoss` and
/// `NetIncomeLossAvailableToCommonStockholdersBasic`.
pub const SIMON_PROPERTY_GROUP: &str = include_str!("CIK0001063761.json");

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::Value;

    use super::*;

    fn parse(fixture: &str) -> Value {
        serde_json::from_str(fixture)
            .expect("Each Company Facts fixture is JSON from the SEC and should always parse")
    }

    fn has_us_gaap_concept(fixture: &str, concept: &str) -> bool {
        parse(fixture)["facts"]["us-gaap"].get(concept).is_some()
    }

    #[test]
    fn should_hold_the_jpmorgan_chase_cik_for_the_jpmorgan_chase_fixture() {
        let fixture = JPMORGAN_CHASE;

        let expected_result = Some(19_617);

        let result = parse(fixture)["cik"].as_u64();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_the_progressive_cik_for_the_progressive_fixture() {
        let fixture = PROGRESSIVE;

        let expected_result = Some(80_661);

        let result = parse(fixture)["cik"].as_u64();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_the_walmart_cik_for_the_walmart_fixture() {
        let fixture = WALMART;

        let expected_result = Some(104_169);

        let result = parse(fixture)["cik"].as_u64();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hold_the_simon_property_group_cik_for_the_simon_property_group_fixture() {
        let fixture = SIMON_PROPERTY_GROUP;

        let expected_result = Some(1_063_761);

        let result = parse(fixture)["cik"].as_u64();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_lack_operating_income_for_the_jpmorgan_chase_fixture() {
        let fixture = JPMORGAN_CHASE;

        let expected_result = false;

        let result = has_us_gaap_concept(fixture, "OperatingIncomeLoss");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_lack_operating_income_for_the_progressive_fixture() {
        let fixture = PROGRESSIVE;

        let expected_result = false;

        let result = has_us_gaap_concept(fixture, "OperatingIncomeLoss");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_lack_total_liabilities_for_the_walmart_fixture() {
        let fixture = WALMART;

        let expected_result = false;

        let result = has_us_gaap_concept(fixture, "Liabilities");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_lack_net_income_for_the_simon_property_group_fixture() {
        let fixture = SIMON_PROPERTY_GROUP;

        let expected_result = false;

        let result = has_us_gaap_concept(fixture, "NetIncomeLoss");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_end_fiscal_year_2025_on_january_31_for_the_walmart_fixture() {
        let fixture = parse(WALMART);

        let expected_result = Some("2025-01-31");

        let result = fixture["facts"]["us-gaap"]["Assets"]["units"]["USD"]
            .as_array()
            .expect("The Walmart fixture reports Assets in USD")
            .iter()
            .filter(|fact| fact["form"] == "10-K" && fact["fy"] == 2025 && fact["fp"] == "FY")
            .filter_map(|fact| fact["end"].as_str())
            .max();

        assert_eq!(result, expected_result);
    }
}
