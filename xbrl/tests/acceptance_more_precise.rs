//! # Acceptance Test A11: The More Precise Copy
//!
//! Runs `core/dedup/more-precise@1` on the Procter & Gamble 10-K for fiscal year 2026. The Notes
//! rows report the restructuring charges of the year three times: twice as 1,230 million USD with
//! `decimals` −6, and once as 1,200 million USD with `decimals` −8. The parser drops the row that
//! repeats a fact, so two copies remain. They agree after rounding to −8, so the rule keeps rank 0,
//! the more precise value.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;
use xbrl::core::dedup::more_precise;
use xbrl::core::fact::{AsIsFact, Decimals, FactValue};
use xbrl::tests::fixtures::data::notes::parse_notes_fixture_filing;
use xbrl::tests::fixtures::sample_fact::date;

/// The as-is concept of the restructuring charges.
const RESTRUCTURING_CHARGES: &str = "RestructuringCharges";

/// The length of a fiscal year, in quarters.
const YEAR_LENGTH: u16 = 4;

/// Returns `true` if `fact` is the restructuring charges of fiscal year 2026, without dimensions.
fn is_restructuring_charges_of_2026(fact: &AsIsFact) -> bool {
    fact.concept.local_name() == RESTRUCTURING_CHARGES
        && fact.context.period.end() == date(2026, 6, 30)
        && fact.context.period.quarters() == YEAR_LENGTH
        && fact.context.dimensions.is_empty()
}

#[test]
fn should_keep_the_more_precise_copy_of_the_restructuring_charges_for_procter_and_gamble_10_k_2026_fy()
 {
    let facts = parse_notes_fixture_filing("0000080424-26-000103").facts;
    let copies: Vec<(u16, FactValue, Option<Decimals>)> = facts
        .iter()
        .filter(|fact| is_restructuring_charges_of_2026(fact))
        .map(|fact| (fact.copy_rank, fact.value.clone(), fact.decimals))
        .collect();

    let expected_result = (
        vec![
            (
                0,
                FactValue::Numeric(BigDecimal::from(1_230_000_000_i64)),
                Some(Decimals::Finite(-6)),
            ),
            (
                1,
                FactValue::Numeric(BigDecimal::from(1_200_000_000_i64)),
                Some(Decimals::Finite(-8)),
            ),
        ],
        vec![(0, FactValue::Numeric(BigDecimal::from(1_230_000_000_i64)))],
    );

    let result = (
        copies,
        more_precise(&facts)
            .facts()
            .iter()
            .filter(|fact| is_restructuring_charges_of_2026(fact))
            .map(|fact| (fact.copy_rank, fact.value.clone()))
            .collect::<Vec<(u16, FactValue)>>(),
    );

    assert_eq!(result, expected_result);
}
