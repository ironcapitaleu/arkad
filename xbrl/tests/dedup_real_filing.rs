//! # Dedup Real Filing Integration Tests
//!
//! Runs the dedup rules on the as-is facts of one real filing, the 10-Q `0001628280-26-068246`,
//! as [`xbrl::adapter::edgar::notes::parse`] and [`xbrl::adapter::edgar::instance::parse`] give
//! them.
//!
//! The filing reports 99 value keys twice. Each pair holds a quarter of three months from 1 July
//! and a period of four months from 1 June. Both round to 1 quarter, so they share a value key (see
//! `DESIGN.md`, "Value Key"). The quarter is rank 0, because its length is closer to a full
//! quarter. The four months hold June plus the quarter, so no pair agrees, and all 99 keys are
//! conflicts.
//!
//! The fake plug-in holds the revenue alias list of the fixtures until the US GAAP plug-in has
//! its own lists.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

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
use xbrl::core::fact::{AsIsFact, Dimensions, FactKey, TaxonomyVersion};
use xbrl::core::filing::{EntityId, FilingId};
use xbrl::core::finding::Finding;
use xbrl::core::taxonomy::registry::Registry;
use xbrl::core::unit::Unit;
use xbrl::tests::fixtures::data::instance::parse_instance_fixture;
use xbrl::tests::fixtures::data::notes::parse_notes_fixture;
use xbrl::tests::fixtures::sample_fact::{date, revenue_aliases};
use xbrl::tests::fixtures::sample_taxonomy::{FakeTaxonomy, declaration, plugin_with_aliases};

/// The as-is concept of revenue in the fixture filing, the third concept of the revenue alias list.
const REVENUE_CONCEPT: &str = "RevenueFromContractWithCustomerExcludingAssessedTax";

/// Returns a plug-in that declares the families of the fixture filing, as `fixture_taxonomy` does,
/// and holds the revenue alias list.
fn revenue_plugin() -> FakeTaxonomy {
    plugin_with_aliases(
        "us-gaap/2026",
        vec![
            declaration("us-gaap", "http://fasb.org/us-gaap/{release}", true),
            declaration("dei", "http://xbrl.sec.gov/dei/{release}", true),
            declaration("srt", "http://fasb.org/srt/{release}", true),
            declaration("ecd", "http://xbrl.sec.gov/ecd/{release}", true),
        ],
        vec![revenue_aliases()],
    )
}

/// Runs `core/dedup/more-precise@1` and tiers 1 and 2 on `facts` with [`revenue_plugin`].
fn candidates_of(facts: &[AsIsFact]) -> FilingCandidates {
    let plugin = revenue_plugin();
    let registry = Registry::new(vec![&plugin]).expect(
        "Given one plug-in with four distinct families, the creation should always succeed",
    );
    resolve_aliases(&more_precise(facts), &registry)
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

fn dimensions(pairs: &[(&str, &str)]) -> Dimensions {
    Dimensions::new(pairs.iter().copied())
        .expect("Given hardcoded pairs with distinct axes, the creation should always succeed")
}

/// Returns the series key of revenue in USD for the quarter that ends on `period_end`.
fn revenue_quarter(period_end: NaiveDate, pairs: &[(&str, &str)]) -> SeriesKey {
    SeriesKey {
        entity: fedex_freight(),
        concept: CanonicalElement::Revenue,
        period_end,
        quarters: 1,
        unit: usd(),
        dimensions: dimensions(pairs),
    }
}

#[test]
fn should_find_99_conflicts_when_more_precise_runs_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    // 99 keys hold two ranks, and none of the 99 pairs agrees.
    let expected_result = 99;

    let result = more_precise(&facts).conflicts().len();

    assert_eq!(result, expected_result);
}

#[test]
fn should_find_99_conflicts_when_more_precise_runs_on_the_instance_facts() {
    let facts = parse_instance_fixture().facts;

    // 99 keys hold two ranks, and none of the 99 pairs agrees.
    let expected_result = 99;

    let result = more_precise(&facts).conflicts().len();

    assert_eq!(result, expected_result);
}

#[test]
fn should_report_the_quarter_as_rank_0_when_revenue_of_the_quarter_and_the_four_months_conflict() {
    let facts = parse_instance_fixture().facts;
    let filing = FilingId::new("sec", "0001628280-26-068246")
        .expect("Given the hardcoded ID of the fixture filing, the creation should always succeed");
    let family = NamespaceFamily::new("us-gaap")
        .expect("Given a hardcoded family without whitespace, the creation should always succeed");

    // Rank 0 is the quarter from 2026-07-01, 2,442 million. Rank 1 is the four months from
    // 2026-06-01, 3,281 million.
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
fn should_give_the_june_revenue_when_the_dedup_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    // Only the month of June has no conflict. It lasts under 45 days, so its length is 0
    // quarters. Its concept is the third of the alias list, so its confidence is a synonym.
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

    let Series { facts: revenue, .. } = newest_filing(&candidates_of(&facts).candidates);
    let result: Vec<(NaiveDate, u16, BigDecimal, Confidence)> = revenue
        .into_iter()
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

    // The revenue concept has a conflict in each quarter, so each quarter gets no revenue.
    let expected_result = vec![
        revenue_quarter(date(2025, 9, 30), &[]),
        revenue_quarter(
            date(2025, 9, 30),
            &[("BusinessSegments", "ReportableSegment")],
        ),
        revenue_quarter(date(2025, 9, 30), &[("ProductOrService", "Economy")]),
        revenue_quarter(date(2025, 9, 30), &[("ProductOrService", "Other")]),
        revenue_quarter(date(2025, 9, 30), &[("ProductOrService", "Priority")]),
        revenue_quarter(
            date(2025, 9, 30),
            &[("RelatedPartyTransactionsByRelatedParty", "RelatedParty")],
        ),
        revenue_quarter(date(2026, 9, 30), &[]),
        revenue_quarter(
            date(2026, 9, 30),
            &[("BusinessSegments", "ReportableSegment")],
        ),
        revenue_quarter(date(2026, 9, 30), &[("ProductOrService", "Economy")]),
        revenue_quarter(date(2026, 9, 30), &[("ProductOrService", "Other")]),
        revenue_quarter(date(2026, 9, 30), &[("ProductOrService", "Priority")]),
        revenue_quarter(
            date(2026, 9, 30),
            &[("RelatedPartyTransactionsByRelatedParty", "RelatedParty")],
        ),
    ];

    let result = newest_filing(&candidates_of(&facts).candidates).blocked;

    assert_eq!(result, expected_result);
}
