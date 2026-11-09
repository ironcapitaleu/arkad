//! # Dedup Real Filing Integration Tests
//!
//! Runs the dedup rules on the as-is facts of one real filing, the 10-Q `0001628280-26-068246`,
//! as [`xbrl::adapter::edgar::notes::parse`] and [`xbrl::adapter::edgar::instance::parse`] give
//! them.
//!
//! The filing reports 99 value keys twice. Each pair holds a quarter of three months from 1 July
//! and a period of four months from 1 June. Both round to 1 quarter, so they share a value key. The
//! quarter is rank 0 in both sources, as the revenue tests show. The instance parser ranks the
//! length nearer to a full quarter first, and the Notes parser follows the order of `iprx` (see
//! `DESIGN.md`, "Value Key"). The four months hold June plus the quarter, so no pair agrees, and
//! all 99 keys are conflicts.
//!
//! The tests that resolve use [`UsGaap`] and its alias lists. They check revenue, and the count of
//! standardized facts per standard concept.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use pretty_assertions::assert_eq;
use xbrl::core::alias::{FilingCandidates, resolve_aliases};
use xbrl::core::candidate::SeriesKey;
use xbrl::core::concept::{ConceptName, NamespaceFamily};
use xbrl::core::confidence::Confidence;
use xbrl::core::dedup::{Series, more_precise, newest_filing};
use xbrl::core::elements::CanonicalElement;
use xbrl::core::fact::{AsIsFact, Dimensions, FactKey, FactValue, TaxonomyVersion};
use xbrl::core::filing::{EntityId, FilingId};
use xbrl::core::finding::Finding;
use xbrl::core::rule::RuleId;
use xbrl::core::taxonomy::registry::Registry;
use xbrl::core::unit::Unit;
use xbrl::tests::fixtures::data::instance::parse_instance_fixture;
use xbrl::tests::fixtures::data::notes::parse_notes_fixture;
use xbrl::tests::fixtures::sample_fact::date;
use xbrl::us_gaap::UsGaap;

/// The as-is concept of revenue in the fixture filing, the first part of the revenue part rule.
const REVENUE_CONCEPT: &str = "RevenueFromContractWithCustomerExcludingAssessedTax";

/// Runs `core/dedup/more-precise@1` and tiers 1 and 2 on `facts` with [`UsGaap`].
fn candidates_of(facts: &[AsIsFact]) -> FilingCandidates {
    let plugin = UsGaap::new();
    let registry = Registry::new(vec![&plugin])
        .expect("Given one plug-in with distinct families, the creation should always succeed");
    resolve_aliases(&more_precise(facts), &registry)
}

/// Returns the rank, period, and value of each copy of revenue without dimensions in the quarter
/// that ends on 2026-09-30, or an empty list if that key has no conflict.
fn revenue_ranks(facts: &[AsIsFact]) -> Vec<(u16, String, FactValue)> {
    more_precise(facts)
        .conflicts()
        .iter()
        .find(|ranks| {
            ranks[0].concept.local_name() == REVENUE_CONCEPT
                && ranks[0].context.period.end() == date(2026, 9, 30)
                && ranks[0].context.dimensions == Dimensions::default()
        })
        .map(|ranks| {
            ranks
                .iter()
                .map(|fact| {
                    (
                        fact.copy_rank,
                        fact.context.period.to_string(),
                        fact.value.clone(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Runs the dedup rules on `facts`, and returns the count of standardized facts per standard
/// concept.
fn standardized_counts(facts: &[AsIsFact]) -> BTreeMap<CanonicalElement, usize> {
    let mut counts = BTreeMap::new();
    for fact in newest_filing(&candidates_of(facts).candidates).facts {
        *counts.entry(fact.concept).or_insert(0) += 1;
    }
    counts
}

/// The count of standardized facts per standard concept in the fixture filing, for both sources.
///
/// Equity has 3 facts, and interest expense has 1, because a conflict blocks the 2026 quarter.
/// Every other concept has 2. Facts with dimensions give no standardized fact.
fn expected_standardized_counts() -> BTreeMap<CanonicalElement, usize> {
    BTreeMap::from([
        (CanonicalElement::Assets, 2),
        (CanonicalElement::Equity, 3),
        (CanonicalElement::Revenue, 2),
        (CanonicalElement::NetIncome, 2),
        (CanonicalElement::OperatingCashFlow, 2),
        (CanonicalElement::InvestingCashFlow, 2),
        (CanonicalElement::FinancingCashFlow, 2),
        (CanonicalElement::CurrentAssets, 2),
        (CanonicalElement::CashAndEquivalents, 2),
        (CanonicalElement::Goodwill, 2),
        (CanonicalElement::CurrentLiabilities, 2),
        (CanonicalElement::NonCurrentLiabilities, 2),
        (CanonicalElement::RetainedEarnings, 2),
        (CanonicalElement::OperatingIncome, 2),
        (CanonicalElement::InterestExpense, 1),
        (CanonicalElement::EarningsPerShareBasic, 2),
        (CanonicalElement::EarningsPerShareDiluted, 2),
        (CanonicalElement::DepreciationAmortization, 2),
        (CanonicalElement::CapitalExpenditures, 2),
        (CanonicalElement::ShareBasedCompensation, 2),
    ])
}

fn rule(text: &str) -> RuleId {
    RuleId::from_str(text)
        .expect("Given a hardcoded rule ID in the documented form, the parse should always succeed")
}

fn number(text: &str) -> BigDecimal {
    BigDecimal::from_str(text)
        .expect("Given a hardcoded decimal text, the parse should always succeed")
}

fn usd() -> Unit {
    Unit::from_str("USD")
        .expect("Given a hardcoded unit with one measure, the parse should always succeed")
}

fn fedex_freight() -> EntityId {
    EntityId::new("http://www.sec.gov/CIK", "0002082247").expect(
        "Given the hardcoded scheme and CIK of the filer, the creation should always succeed",
    )
}

/// Returns the series key of revenue in USD without dimensions for the quarter that ends on
/// `period_end`.
fn revenue_quarter(period_end: NaiveDate) -> SeriesKey {
    SeriesKey {
        entity: fedex_freight(),
        concept: CanonicalElement::Revenue,
        period_end,
        quarters: 1,
        unit: usd(),
        dimensions: Dimensions::default(),
    }
}

#[test]
fn should_find_99_conflicts_when_more_precise_runs_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    // 613 facts: 415 keys with one rank, and 99 keys with two ranks that disagree. 415 + 2 x 99 =
    // 613, so no rank agrees with rank 0 and drops out.
    let expected_result = (415, 99);

    let copies = more_precise(&facts);
    let result = (copies.facts().len(), copies.conflicts().len());

    assert_eq!(result, expected_result);
}

#[test]
fn should_find_99_conflicts_when_more_precise_runs_on_the_instance_facts() {
    let facts = parse_instance_fixture().facts;

    // 613 facts: 415 keys with one rank, and 99 keys with two ranks that disagree. 415 + 2 x 99 =
    // 613, so no rank agrees with rank 0 and drops out.
    let expected_result = (415, 99);

    let copies = more_precise(&facts);
    let result = (copies.facts().len(), copies.conflicts().len());

    assert_eq!(result, expected_result);
}

#[test]
fn should_rank_the_quarter_before_the_four_months_when_revenue_of_both_shares_a_value_key_in_the_instance()
 {
    let facts = parse_instance_fixture().facts;

    let expected_result = vec![
        (
            0,
            "2026-07-01 to 2026-09-30".to_string(),
            FactValue::Numeric(number("2442000000")),
        ),
        (
            1,
            "2026-06-01 to 2026-09-30".to_string(),
            FactValue::Numeric(number("3281000000")),
        ),
    ];

    let result = revenue_ranks(&facts);

    assert_eq!(result, expected_result);
}

#[test]
fn should_rank_the_quarter_before_the_four_months_when_revenue_of_both_shares_a_value_key_in_notes()
{
    let facts = parse_notes_fixture().facts;

    // Notes gives no exact start, so only the value tells the quarter from the four months.
    let expected_result = vec![
        (
            0,
            "1 quarters to 2026-09-30".to_string(),
            FactValue::Numeric(number("2442000000")),
        ),
        (
            1,
            "1 quarters to 2026-09-30".to_string(),
            FactValue::Numeric(number("3281000000")),
        ),
    ];

    let result = revenue_ranks(&facts);

    assert_eq!(result, expected_result);
}

#[test]
fn should_report_rank_0_as_key_and_rank_1_as_disagreeing_when_revenue_of_the_quarter_has_a_conflict()
 {
    let facts = parse_instance_fixture().facts;
    let filing = FilingId::new("sec", "0001628280-26-068246")
        .expect("Given the hardcoded ID of the fixture filing, the creation should always succeed");
    let family = NamespaceFamily::new("us-gaap")
        .expect("Given a hardcoded family without whitespace, the creation should always succeed");

    let expected_result = Some(Finding::Conflict {
        entity: fedex_freight(),
        key: FactKey {
            filing,
            concept: ConceptName::new(family, REVENUE_CONCEPT).expect(
                "Given a hardcoded local name without a colon, the creation should always succeed",
            ),
            taxonomy_version: Some(
                TaxonomyVersion::new("us-gaap/2026")
                    .expect("Given a hardcoded version, the creation should always succeed"),
            ),
            period_end: date(2026, 9, 30),
            quarters: 1,
            unit: Some(usd()),
            dimensions: Dimensions::default(),
            copy_rank: 0,
        },
        ranks: vec![1],
        standard_concept: Some(CanonicalElement::Revenue),
    });

    let result = candidates_of(&facts).findings.into_iter().find(|finding| {
        matches!(
            finding,
            Finding::Conflict { key, .. }
                if key.concept.local_name() == REVENUE_CONCEPT
                    && key.period_end == date(2026, 9, 30)
                    && key.dimensions == Dimensions::default()
        )
    });

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_a_conflict_finding_for_each_conflict_with_dimensions_when_tiers_1_and_2_run_on_the_instance_facts()
 {
    let facts = parse_instance_fixture().facts;

    // 52 of the 99 conflicts are on facts with dimensions. Each still gives a finding.
    let expected_result = 52;

    let result = candidates_of(&facts)
        .findings
        .iter()
        .filter(|finding| {
            matches!(finding, Finding::Conflict { key, .. } if !key.dimensions.is_empty())
        })
        .count();

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_the_june_revenue_when_the_dedup_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    // Only the month of June has no conflict. June lasts 30 days, so its length rounds to 0
    // quarters. Its concept is a part of revenue, so its confidence is a synonym.
    let expected_result = vec![
        (
            date(2025, 6, 30),
            0,
            number("745000000"),
            Confidence::Synonym,
        ),
        (
            date(2026, 6, 30),
            0,
            number("839000000"),
            Confidence::Synonym,
        ),
    ];

    let Series {
        facts: standardized,
        ..
    } = newest_filing(&candidates_of(&facts).candidates);
    let result: Vec<(NaiveDate, u16, BigDecimal, Confidence)> = standardized
        .into_iter()
        .filter(|fact| fact.concept == CanonicalElement::Revenue)
        .map(|fact| {
            (
                fact.period.end(),
                fact.period.quarters(),
                fact.value,
                fact.confidence,
            )
        })
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_block_revenue_of_each_quarter_when_the_dedup_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    // The revenue concept has a conflict in each quarter, so each quarter gets no revenue. The
    // segment members of revenue have conflicts too, but a fact with dimensions gives no candidate.
    let expected_result = vec![
        revenue_quarter(date(2025, 9, 30)),
        revenue_quarter(date(2026, 9, 30)),
    ];

    let result: Vec<SeriesKey> = newest_filing(&candidates_of(&facts).candidates)
        .blocked
        .into_iter()
        .filter(|key| key.concept == CanonicalElement::Revenue)
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_40_standardized_facts_in_20_concepts_when_the_dedup_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    let expected_result = expected_standardized_counts();

    let result = standardized_counts(&facts);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_40_standardized_facts_in_20_concepts_when_the_dedup_rules_run_on_the_instance_facts()
{
    let facts = parse_instance_fixture().facts;

    let expected_result = expected_standardized_counts();

    let result = standardized_counts(&facts);

    assert_eq!(result, expected_result);
}

#[test]
fn should_resolve_five_concepts_by_their_part_rules_when_the_dedup_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    // For revenue, net income, equity, cash, and interest expense, the filer reports a part and no
    // other part.
    let expected_result = BTreeSet::from([
        (CanonicalElement::Assets, rule("us-gaap/alias/assets@1")),
        (CanonicalElement::Equity, rule("us-gaap/part/equity@1")),
        (CanonicalElement::Revenue, rule("us-gaap/part/revenue@1")),
        (
            CanonicalElement::NetIncome,
            rule("us-gaap/part/net-income@1"),
        ),
        (
            CanonicalElement::OperatingCashFlow,
            rule("us-gaap/alias/operating-cash-flow@1"),
        ),
        (
            CanonicalElement::InvestingCashFlow,
            rule("us-gaap/alias/investing-cash-flow@1"),
        ),
        (
            CanonicalElement::FinancingCashFlow,
            rule("us-gaap/alias/financing-cash-flow@1"),
        ),
        (
            CanonicalElement::CurrentAssets,
            rule("us-gaap/alias/current-assets@1"),
        ),
        (
            CanonicalElement::CashAndEquivalents,
            rule("us-gaap/part/cash-and-equivalents@1"),
        ),
        (CanonicalElement::Goodwill, rule("us-gaap/alias/goodwill@1")),
        (
            CanonicalElement::CurrentLiabilities,
            rule("us-gaap/alias/current-liabilities@1"),
        ),
        (
            CanonicalElement::NonCurrentLiabilities,
            rule("us-gaap/alias/non-current-liabilities@1"),
        ),
        (
            CanonicalElement::RetainedEarnings,
            rule("us-gaap/alias/retained-earnings@1"),
        ),
        (
            CanonicalElement::OperatingIncome,
            rule("us-gaap/alias/operating-income@1"),
        ),
        (
            CanonicalElement::InterestExpense,
            rule("us-gaap/part/interest-expense@1"),
        ),
        (
            CanonicalElement::EarningsPerShareBasic,
            rule("us-gaap/alias/earnings-per-share-basic@1"),
        ),
        (
            CanonicalElement::EarningsPerShareDiluted,
            rule("us-gaap/alias/earnings-per-share-diluted@1"),
        ),
        (
            CanonicalElement::DepreciationAmortization,
            rule("us-gaap/alias/depreciation-amortization@1"),
        ),
        (
            CanonicalElement::CapitalExpenditures,
            rule("us-gaap/alias/capital-expenditures@1"),
        ),
        (
            CanonicalElement::ShareBasedCompensation,
            rule("us-gaap/alias/share-based-compensation@1"),
        ),
    ]);

    let result: BTreeSet<(CanonicalElement, RuleId)> =
        newest_filing(&candidates_of(&facts).candidates)
            .facts
            .into_iter()
            .map(|fact| (fact.concept, fact.rule))
            .collect();

    assert_eq!(result, expected_result);
}
