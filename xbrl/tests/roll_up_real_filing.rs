//! # Roll-Up Real Filing Integration Tests
//!
//! Runs the derivation from a closed group on the standardized facts of one real filing, the 10-Q
//! `0001628280-26-068246`, as [`xbrl::adapter::edgar::notes::parse`] and
//! [`xbrl::adapter::edgar::instance::parse`] give them.
//!
//! The filing reports no `Liabilities` and no `AssetsNoncurrent` on its balance sheets of
//! 2026-05-31 and 2026-09-30. It reports no temporary equity and no noncontrolling interest, so both
//! count as zero.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;
use xbrl::core::alias::resolve_aliases;
use xbrl::core::closed_group::derive_from_closed_groups;
use xbrl::core::dedup::{more_precise, newest_filing};
use xbrl::core::elements::{BalanceSheetConcept, CanonicalElementKind};
use xbrl::core::fact::AsIsFact;
use xbrl::core::period_identity::derive_q4;
use xbrl::core::taxonomy::registry::Registry;
use xbrl::tests::fixtures::data::instance::parse_instance_fixture;
use xbrl::tests::fixtures::data::notes::parse_notes_fixture;
use xbrl::us_gaap::UsGaap;

/// The concept, the period, the value, and the rule of a derived fact.
type Summary = (CanonicalElementKind, String, BigDecimal, String);

/// Runs the dedup rules, tiers 1 and 2 with [`UsGaap`], `core/period/q4-from-fy@1`, and the
/// derivation from a closed group on `facts`, and returns the derived facts of the last step.
fn roll_ups(facts: &[AsIsFact]) -> Vec<Summary> {
    let plugin = UsGaap::new();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with distinct families, the creation should always succeed");
    let series = newest_filing(&resolve_aliases(&more_precise(facts), &registry).candidates);
    let fourth_quarters = derive_q4(&series, &[]);
    derive_from_closed_groups(&series, &fourth_quarters)
        .into_iter()
        .map(|fact| {
            (
                fact.concept,
                fact.period.to_string(),
                fact.value,
                fact.rule.to_string(),
            )
        })
        .collect()
}

/// The derived facts of the fixture filing, in millions of USD: `NonCurrentAssets` from `assets`,
/// then `Liabilities` from `liabilities-and-equity`.
fn expected_roll_ups() -> Vec<Summary> {
    vec![
        (
            BalanceSheetConcept::NonCurrentAssets.element(),
            "2026-05-31".to_string(),
            BigDecimal::from(5_378_000_000_i64),
            "core/roll-up/assets@1".to_string(),
        ),
        (
            BalanceSheetConcept::NonCurrentAssets.element(),
            "2026-09-30".to_string(),
            BigDecimal::from(5_370_000_000_i64),
            "core/roll-up/assets@1".to_string(),
        ),
        (
            BalanceSheetConcept::Liabilities.element(),
            "2026-05-31".to_string(),
            BigDecimal::from(7_381_000_000_i64),
            "core/roll-up/liabilities-and-equity@1".to_string(),
        ),
        (
            BalanceSheetConcept::Liabilities.element(),
            "2026-09-30".to_string(),
            BigDecimal::from(7_468_000_000_i64),
            "core/roll-up/liabilities-and-equity@1".to_string(),
        ),
    ]
}

#[test]
fn should_derive_liabilities_and_non_current_assets_when_the_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    let expected_result = expected_roll_ups();

    let result = roll_ups(&facts);

    assert_eq!(result, expected_result);
}

#[test]
fn should_derive_liabilities_and_non_current_assets_when_the_rules_run_on_the_instance_facts() {
    let facts = parse_instance_fixture().facts;

    let expected_result = expected_roll_ups();

    let result = roll_ups(&facts);

    assert_eq!(result, expected_result);
}
