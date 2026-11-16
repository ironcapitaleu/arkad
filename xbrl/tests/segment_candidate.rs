//! # Segment Candidate Integration Tests
//!
//! Runs `core/dedup/more-precise@1` and tiers 1 and 2 with [`UsGaap`] on facts with dimensions. A
//! registry that holds [`UsGaap`] as [`Plugin::WithSegmentAxes`] gives a candidate for each
//! segment fact. A registry from [`Registry::new`] gives none.
//!
//! ## External Dependencies
//!
//! None. The tests build their facts in code.

use std::str::FromStr;

use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;
use xbrl::core::alias::resolve_aliases;
use xbrl::core::candidate::CandidateValue;
use xbrl::core::confidence::Confidence;
use xbrl::core::dedup::more_precise;
use xbrl::core::elements::{CanonicalElementKind, IncomeStatementConcept};
use xbrl::core::fact::{AsIsFact, Decimals, Dimensions, FactValue};
use xbrl::core::rule::RuleId;
use xbrl::core::taxonomy::registry::{Plugin, Registry};
use xbrl::tests::fixtures::sample_fact::{date, fact, source};
use xbrl::us_gaap::UsGaap;

/// The standard concept, the rule, the dimensions, and the value of a candidate.
type Summary = (CanonicalElementKind, RuleId, Dimensions, CandidateValue);

/// Runs `core/dedup/more-precise@1` and tiers 1 and 2 on `facts`, and returns the summary of each
/// candidate.
fn resolve(facts: &[AsIsFact], registry: &Registry<'_>) -> Vec<Summary> {
    resolve_aliases(&more_precise(facts), registry)
        .candidates
        .into_iter()
        .map(|candidate| {
            (
                candidate.concept,
                candidate.rule,
                candidate.dimensions,
                candidate.value,
            )
        })
        .collect()
}

/// Returns the fact `us-gaap:<local_name>` = `value` with the axis and member pairs `pairs`.
fn fact_with(local_name: &str, value: &str, pairs: &[(&str, &str)]) -> AsIsFact {
    let filing = source("0000320193-24-000123", date(2024, 11, 1));
    let mut fact = fact(&filing, local_name, value);
    fact.context.dimensions = dimensions(pairs);
    fact
}

fn dimensions(pairs: &[(&str, &str)]) -> Dimensions {
    Dimensions::new(pairs.iter().copied())
        .expect("Given hardcoded axis and member pairs, the creation should always succeed")
}

fn rule(text: &str) -> RuleId {
    RuleId::from_str(text)
        .expect("Given a hardcoded rule ID in the documented form, the parse should always succeed")
}

/// Returns the candidate value `value` at `decimals` -6 and confidence [`Confidence::Exact`].
fn exact(value: i64) -> CandidateValue {
    CandidateValue::Reported {
        value: BigDecimal::from(value),
        decimals: Some(Decimals::Finite(-6)),
        confidence: Confidence::Exact,
    }
}

#[test]
fn should_give_a_revenue_candidate_with_the_segment_when_revenues_carry_the_cloud_segment() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let facts = vec![fact_with(
        "Revenues",
        "24000000000",
        &[("BusinessSegments", "Cloud")],
    )];

    let expected_result = vec![(
        IncomeStatementConcept::Revenue.element(),
        rule("us-gaap/alias/revenue@1"),
        dimensions(&[("BusinessSegments", "Cloud")]),
        exact(24_000_000_000),
    )];

    let result = resolve(&facts, &registry);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_no_candidate_when_the_fact_carries_a_legal_entity() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let facts = vec![fact_with(
        "Revenues",
        "24000000000",
        &[("LegalEntity", "OhioPowerCompany")],
    )];

    let expected_result: Vec<Summary> = Vec::new();

    let result = resolve(&facts, &registry);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_no_candidate_when_the_fact_carries_a_debt_instrument() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let facts = vec![fact_with(
        "InterestExpense",
        "100000000",
        &[("DebtInstrument", "SeniorNotesDue2030")],
    )];

    let expected_result: Vec<Summary> = Vec::new();

    let result = resolve(&facts, &registry);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_no_candidate_when_one_of_two_axes_is_no_segment_axis() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let facts = vec![fact_with(
        "Revenues",
        "24000000000",
        &[
            ("BusinessSegments", "Cloud"),
            ("LegalEntity", "OhioPowerCompany"),
        ],
    )];

    let expected_result: Vec<Summary> = Vec::new();

    let result = resolve(&facts, &registry);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_no_candidate_when_the_registry_holds_the_plugin_without_segment_axes() {
    let us_gaap = UsGaap::new();
    let registry = Registry::new(vec![&us_gaap])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let facts = vec![fact_with(
        "Revenues",
        "24000000000",
        &[("BusinessSegments", "Cloud")],
    )];

    let expected_result: Vec<Summary> = Vec::new();

    let result = resolve(&facts, &registry);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_no_candidate_when_a_per_share_concept_carries_the_cloud_segment() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let mut eps = fact_with(
        "EarningsPerShareBasic",
        "1.25",
        &[("BusinessSegments", "Cloud")],
    );
    eps.decimals = Some(Decimals::Finite(2));
    let facts = vec![eps];

    let expected_result: Vec<Summary> = Vec::new();

    let result = resolve(&facts, &registry);

    assert_eq!(result, expected_result);
}

#[test]
fn should_block_only_the_segment_candidate_when_the_segment_fact_has_a_conflict() {
    let us_gaap = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&us_gaap)])
        .expect("Given one plug-in with distinct families, the registry should always start");
    let cloud = [("BusinessSegments", "Cloud")];
    let quarter = fact_with("Revenues", "24000000000", &cloud);
    let mut stub = quarter.clone();
    stub.value = FactValue::Numeric(BigDecimal::from(16_000_000_000_i64));
    stub.copy_rank = 1;
    let facts = vec![fact_with("Revenues", "391035000000", &[]), quarter, stub];

    let expected_result = vec![
        (
            IncomeStatementConcept::Revenue.element(),
            rule("us-gaap/alias/revenue@1"),
            Dimensions::default(),
            exact(391_035_000_000),
        ),
        (
            IncomeStatementConcept::Revenue.element(),
            rule("us-gaap/alias/revenue@1"),
            dimensions(&cloud),
            CandidateValue::Blocked,
        ),
    ];

    let result = resolve(&facts, &registry);

    assert_eq!(result, expected_result);
}
