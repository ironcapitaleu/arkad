//! # Acceptance Test A11: The More Precise Copy
//!
//! Runs `core/dedup/more-precise@1` on the Procter & Gamble 10-K for fiscal year 2026. The filing
//! reports the restructuring charges of the year three times: twice as 1,230 million USD with
//! `decimals` −6, and once as 1,200 million USD with `decimals` −8. The copies agree after rounding
//! to −8, so the rule keeps rank 0, the more precise value.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;
use xbrl::core::dedup::more_precise;
use xbrl::core::fact::FactValue;
use xbrl::tests::fixtures::data::notes::parse_notes_fixture_filing;
use xbrl::tests::fixtures::sample_fact::date;

/// The as-is concept of the restructuring charges.
const RESTRUCTURING_CHARGES: &str = "RestructuringCharges";

/// The length of a fiscal year, in quarters.
const YEAR_LENGTH: u16 = 4;

#[test]
fn should_keep_the_more_precise_copy_of_the_restructuring_charges_for_procter_and_gamble_10_k_2026_fy()
 {
    let facts = parse_notes_fixture_filing("0000080424-26-000103").facts;

    let expected_result = vec![(0, FactValue::Numeric(BigDecimal::from(1_230_000_000_i64)))];

    let result: Vec<(u16, FactValue)> = more_precise(&facts)
        .facts()
        .iter()
        .filter(|fact| {
            fact.concept.local_name() == RESTRUCTURING_CHARGES
                && fact.context.period.end() == date(2026, 6, 30)
                && fact.context.period.quarters() == YEAR_LENGTH
                && fact.context.dimensions.is_empty()
        })
        .map(|fact| (fact.copy_rank, fact.value.clone()))
        .collect();

    assert_eq!(result, expected_result);
}
