//! # Dedup Real Filing Integration Tests
//!
//! Runs the dedup rules on the as-is facts of one real filing, the 10-Q `0001628280-26-068246`,
//! as [`xbrl::adapter::edgar::notes::parse`] and [`xbrl::adapter::edgar::instance::parse`] give
//! them.
//!
//! The filing reports 99 value keys twice. Each pair holds a quarter of three months from 1 July
//! and a period of four months from 1 June. Both round to 1 quarter, so they share a value key. The
//! quarter is rank 0 in both sources, as the revenue tests show. The instance parser ranks the
//! length nearer to a full quarter first, and the Notes parser follows the order of `iprx`. The
//! four months hold June plus the quarter, so no pair agrees.
//!
//! Both parsers give each duration its exact start: the instance from `startDate`, Notes from
//! `durp`. So the two ranks of each key are two facts, and no key is a conflict.
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
use xbrl::core::candidate::CandidateValue;
use xbrl::core::confidence::Confidence;
use xbrl::core::dedup::{Series, more_precise, newest_filing};
use xbrl::core::elements::{
    BalanceSheetConcept, CanonicalElementKind, CashFlowConcept, DebtMaturityConcept,
    IncomeStatementConcept, PerShareConcept,
};
use xbrl::core::fact::{AsIsFact, Dimensions, FactValue};
use xbrl::core::finding::Finding;
use xbrl::core::rule::RuleId;
use xbrl::core::taxonomy::registry::{Plugin, Registry};
use xbrl::tests::fixtures::data::instance::parse_instance_fixture;
use xbrl::tests::fixtures::data::notes::parse_notes_fixture;
use xbrl::tests::fixtures::sample_fact::date;
use xbrl::us_gaap::UsGaap;

/// The as-is concept of revenue in the fixture filing, the first part of the revenue part rule.
const REVENUE_CONCEPT: &str = "RevenueFromContractWithCustomerExcludingAssessedTax";

/// Runs `core/dedup/more-precise@1` and tiers 1 and 2 on `facts` with [`UsGaap`].
fn candidates_of(facts: &[AsIsFact]) -> FilingCandidates {
    let plugin = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&plugin)])
        .expect("Given one plug-in with distinct families, the creation should always succeed");
    resolve_aliases(&more_precise(facts), &registry)
}

/// Returns the rank, period, and value of each rank of revenue with `dimensions` whose value key
/// ends on 2026-09-30 and has 1 quarter.
fn revenue_ranks(facts: &[AsIsFact], dimensions: &Dimensions) -> Vec<(u16, String, FactValue)> {
    let mut ranks: Vec<(u16, String, FactValue)> = facts
        .iter()
        .filter(|fact| {
            fact.concept.local_name() == REVENUE_CONCEPT
                && fact.context.period.end() == date(2026, 9, 30)
                && fact.context.period.quarters() == 1
                && fact.context.dimensions == *dimensions
        })
        .map(|fact| {
            (
                fact.copy_rank,
                fact.context.period.to_string(),
                fact.value.clone(),
            )
        })
        .collect();
    ranks.sort_by_key(|(rank, ..)| *rank);
    ranks
}

/// Runs the dedup rules on `facts`, and returns the count of standardized facts without dimensions
/// per standard concept.
fn standardized_counts(facts: &[AsIsFact]) -> BTreeMap<CanonicalElementKind, usize> {
    let mut counts = BTreeMap::new();
    for fact in newest_filing(&candidates_of(facts).candidates)
        .facts
        .into_iter()
        .filter(|fact| fact.dimensions.is_empty())
    {
        *counts.entry(fact.concept).or_insert(0) += 1;
    }
    counts
}

/// The count of standardized facts per standard concept in the Notes facts and in the instance facts
/// of the fixture filing.
///
/// Equity and equity attributable to parent have 3 facts each, because the filing reports
/// `StockholdersEquity` without dimensions at 3 instants: 2025-09-30, 2026-05-31, and 2026-09-30.
/// Both sources give each duration its exact start, so no conflict blocks a quarter. The concepts
/// of the income statement, the earnings per share, and D&A get both quarters next to both months
/// of June, so each has 4. Interest expense gets both quarters and no June, so it has 2. Every
/// other concept has 2.
fn expected_standardized_counts() -> BTreeMap<CanonicalElementKind, usize> {
    BTreeMap::from([
        (BalanceSheetConcept::Assets.element(), 2),
        (BalanceSheetConcept::Equity.element(), 3),
        (IncomeStatementConcept::Revenue.element(), 4),
        (IncomeStatementConcept::NetIncome.element(), 4),
        (CashFlowConcept::OperatingCashFlow.element(), 2),
        (CashFlowConcept::InvestingCashFlow.element(), 2),
        (CashFlowConcept::FinancingCashFlow.element(), 2),
        (BalanceSheetConcept::CurrentAssets.element(), 2),
        (BalanceSheetConcept::CashAndEquivalents.element(), 2),
        (BalanceSheetConcept::Goodwill.element(), 2),
        (BalanceSheetConcept::CurrentLiabilities.element(), 2),
        (BalanceSheetConcept::NonCurrentLiabilities.element(), 2),
        (BalanceSheetConcept::RetainedEarnings.element(), 2),
        (IncomeStatementConcept::OperatingIncome.element(), 4),
        (IncomeStatementConcept::InterestExpense.element(), 2),
        (PerShareConcept::EarningsPerShareBasic.element(), 4),
        (PerShareConcept::EarningsPerShareDiluted.element(), 4),
        (CashFlowConcept::DepreciationAmortization.element(), 4),
        (CashFlowConcept::CapitalExpenditures.element(), 2),
        (CashFlowConcept::ShareBasedCompensation.element(), 2),
        (IncomeStatementConcept::PretaxIncome.element(), 4),
        (IncomeStatementConcept::IncomeTaxExpense.element(), 4),
        (
            IncomeStatementConcept::NetIncomeAttributableToParent.element(),
            4,
        ),
        (CashFlowConcept::NetChangeInCash.element(), 2),
        (CashFlowConcept::ExchangeRateEffect.element(), 2),
        (BalanceSheetConcept::LiabilitiesAndEquity.element(), 2),
        (BalanceSheetConcept::AccountsPayable.element(), 2),
        (
            DebtMaturityConcept::LongTermDebtIncludingCurrentMaturities.element(),
            2,
        ),
        (BalanceSheetConcept::ShortTermDebt.element(), 2),
        (BalanceSheetConcept::EquityAttributableToParent.element(), 3),
        (BalanceSheetConcept::AdditionalPaidInCapital.element(), 2),
        (
            BalanceSheetConcept::AccumulatedOtherComprehensiveIncome.element(),
            2,
        ),
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

/// Returns the dimensions `BusinessSegments` = `ReportableSegment`, the one segment of the filer.
fn reportable_segment() -> Dimensions {
    Dimensions::new([("BusinessSegments", "ReportableSegment")])
        .expect("Given one hardcoded axis and member, the creation should always succeed")
}

#[test]
fn should_find_no_conflict_when_more_precise_runs_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    // 613 facts in 514 keys. The two ranks of each of the 99 keys with two ranks have different
    // exact starts, and the quarter is rank 0, so each key keeps rank 0.
    let expected_result = (514, 0);

    let copies = more_precise(&facts);
    let result = (copies.facts().len(), copies.conflicts().len());

    assert_eq!(result, expected_result);
}

#[test]
fn should_find_no_conflict_when_more_precise_runs_on_the_instance_facts() {
    let facts = parse_instance_fixture().facts;

    // 613 facts in 514 keys. The two ranks of each of the 99 keys with two ranks have different
    // exact starts, and the quarter is rank 0, so each key keeps rank 0.
    let expected_result = (514, 0);

    let copies = more_precise(&facts);
    let result = (copies.facts().len(), copies.conflicts().len());

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_the_3_month_revenue_of_each_quarter_when_the_dedup_rules_run_on_the_instance_facts()
{
    let facts = parse_instance_fixture().facts;

    let expected_result = vec![
        ("2025-07-01 to 2025-09-30".to_string(), number("2259000000")),
        ("2026-07-01 to 2026-09-30".to_string(), number("2442000000")),
    ];

    let result: Vec<(String, BigDecimal)> = newest_filing(&candidates_of(&facts).candidates)
        .facts
        .into_iter()
        .filter(|fact| {
            fact.concept == IncomeStatementConcept::Revenue.element()
                && fact.dimensions.is_empty()
                && fact.period.quarters() == 1
        })
        .map(|fact| (fact.period.to_string(), fact.value))
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_the_3_month_revenue_of_each_quarter_when_the_dedup_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    let expected_result = vec![
        ("2025-07-01 to 2025-09-30".to_string(), number("2259000000")),
        ("2026-07-01 to 2026-09-30".to_string(), number("2442000000")),
    ];

    let result: Vec<(String, BigDecimal)> = newest_filing(&candidates_of(&facts).candidates)
        .facts
        .into_iter()
        .filter(|fact| {
            fact.concept == IncomeStatementConcept::Revenue.element()
                && fact.dimensions.is_empty()
                && fact.period.quarters() == 1
        })
        .map(|fact| (fact.period.to_string(), fact.value))
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_rank_the_quarter_before_the_four_months_when_revenue_of_both_shares_a_value_key_in_notes()
{
    let facts = parse_notes_fixture().facts;

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

    let result = revenue_ranks(&facts, &Dimensions::default());

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_no_conflict_finding_when_tiers_1_and_2_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    let expected_result = 0;

    let result = candidates_of(&facts)
        .findings
        .iter()
        .filter(|finding| matches!(finding, Finding::Conflict { .. }))
        .count();

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_the_june_revenue_when_the_dedup_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    // June lasts 30 days, so its length rounds to 0 quarters. Its concept is a part of revenue, so
    // its confidence is a synonym.
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
        .filter(|fact| {
            fact.concept == IncomeStatementConcept::Revenue.element() && fact.period.quarters() == 0
        })
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
fn should_rank_the_quarter_before_the_four_months_when_revenue_of_the_reportable_segment_shares_a_value_key_in_notes()
 {
    let facts = parse_notes_fixture().facts;
    let segment = reportable_segment();

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

    let result = revenue_ranks(&facts, &segment);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_the_3_month_revenue_candidate_of_the_reportable_segment_when_tiers_1_and_2_run_on_the_notes_facts()
 {
    let facts = parse_notes_fixture().facts;
    let segment = reportable_segment();

    // The four months have another exact start, so the candidate holds the value of the quarter.
    let expected_result = vec![(
        rule("us-gaap/part/revenue@1"),
        "2026-07-01 to 2026-09-30".to_string(),
        Some(number("2442000000")),
    )];

    let result: Vec<(RuleId, String, Option<BigDecimal>)> = candidates_of(&facts)
        .candidates
        .into_iter()
        .filter(|candidate| {
            candidate.concept == IncomeStatementConcept::Revenue.element()
                && candidate.dimensions == segment
                && candidate.period.end() == date(2026, 9, 30)
        })
        .map(|candidate| {
            let value = match candidate.value {
                CandidateValue::Reported { value, .. } => Some(value),
                CandidateValue::Blocked => None,
            };
            (candidate.rule, candidate.period.to_string(), value)
        })
        .collect();

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_84_standardized_facts_in_32_concepts_when_the_dedup_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    let expected_result = expected_standardized_counts();

    let result = standardized_counts(&facts);

    assert_eq!(result, expected_result);
}

#[test]
fn should_give_84_standardized_facts_in_32_concepts_when_the_dedup_rules_run_on_the_instance_facts()
{
    let facts = parse_instance_fixture().facts;

    let expected_result = expected_standardized_counts();

    let result = standardized_counts(&facts);

    assert_eq!(result, expected_result);
}

// The test writes out the rule of every standardized concept of the fixture in one place, so a
// concept that changes its rule fails it.
#[allow(clippy::too_many_lines)]
#[test]
fn should_resolve_seven_concepts_by_their_part_rules_when_the_dedup_rules_run_on_the_notes_facts() {
    let facts = parse_notes_fixture().facts;

    // For revenue, net income, equity, cash, interest expense, short-term debt, and additional
    // paid-in capital, the filer reports a part and no other part.
    let expected_result = BTreeSet::from([
        (
            BalanceSheetConcept::Assets.element(),
            rule("us-gaap/alias/assets@1"),
        ),
        (
            BalanceSheetConcept::Equity.element(),
            rule("us-gaap/part/equity@1"),
        ),
        (
            IncomeStatementConcept::Revenue.element(),
            rule("us-gaap/part/revenue@1"),
        ),
        (
            IncomeStatementConcept::NetIncome.element(),
            rule("us-gaap/part/net-income@1"),
        ),
        (
            CashFlowConcept::OperatingCashFlow.element(),
            rule("us-gaap/alias/operating-cash-flow@1"),
        ),
        (
            CashFlowConcept::InvestingCashFlow.element(),
            rule("us-gaap/alias/investing-cash-flow@1"),
        ),
        (
            CashFlowConcept::FinancingCashFlow.element(),
            rule("us-gaap/alias/financing-cash-flow@1"),
        ),
        (
            BalanceSheetConcept::CurrentAssets.element(),
            rule("us-gaap/alias/current-assets@1"),
        ),
        (
            BalanceSheetConcept::CashAndEquivalents.element(),
            rule("us-gaap/part/cash-and-equivalents@1"),
        ),
        (
            BalanceSheetConcept::Goodwill.element(),
            rule("us-gaap/alias/goodwill@1"),
        ),
        (
            BalanceSheetConcept::CurrentLiabilities.element(),
            rule("us-gaap/alias/current-liabilities@1"),
        ),
        (
            BalanceSheetConcept::NonCurrentLiabilities.element(),
            rule("us-gaap/alias/non-current-liabilities@1"),
        ),
        (
            BalanceSheetConcept::RetainedEarnings.element(),
            rule("us-gaap/alias/retained-earnings@1"),
        ),
        (
            IncomeStatementConcept::OperatingIncome.element(),
            rule("us-gaap/alias/operating-income@1"),
        ),
        (
            IncomeStatementConcept::InterestExpense.element(),
            rule("us-gaap/part/interest-expense@1"),
        ),
        (
            PerShareConcept::EarningsPerShareBasic.element(),
            rule("us-gaap/alias/earnings-per-share-basic@1"),
        ),
        (
            PerShareConcept::EarningsPerShareDiluted.element(),
            rule("us-gaap/alias/earnings-per-share-diluted@1"),
        ),
        (
            CashFlowConcept::DepreciationAmortization.element(),
            rule("us-gaap/alias/depreciation-amortization@1"),
        ),
        (
            CashFlowConcept::CapitalExpenditures.element(),
            rule("us-gaap/alias/capital-expenditures@1"),
        ),
        (
            CashFlowConcept::ShareBasedCompensation.element(),
            rule("us-gaap/alias/share-based-compensation@1"),
        ),
        (
            IncomeStatementConcept::PretaxIncome.element(),
            rule("us-gaap/alias/pretax-income@1"),
        ),
        (
            IncomeStatementConcept::IncomeTaxExpense.element(),
            rule("us-gaap/alias/income-tax-expense@1"),
        ),
        (
            IncomeStatementConcept::NetIncomeAttributableToParent.element(),
            rule("us-gaap/alias/net-income-attributable-to-parent@1"),
        ),
        (
            CashFlowConcept::NetChangeInCash.element(),
            rule("us-gaap/alias/net-change-in-cash@1"),
        ),
        (
            CashFlowConcept::ExchangeRateEffect.element(),
            rule("us-gaap/alias/exchange-rate-effect@1"),
        ),
        (
            BalanceSheetConcept::LiabilitiesAndEquity.element(),
            rule("us-gaap/alias/liabilities-and-equity@1"),
        ),
        (
            BalanceSheetConcept::AccountsPayable.element(),
            rule("us-gaap/alias/accounts-payable@1"),
        ),
        (
            BalanceSheetConcept::ShortTermDebt.element(),
            rule("us-gaap/part/short-term-debt@1"),
        ),
        (
            BalanceSheetConcept::EquityAttributableToParent.element(),
            rule("us-gaap/alias/equity-attributable-to-parent@1"),
        ),
        (
            BalanceSheetConcept::AdditionalPaidInCapital.element(),
            rule("us-gaap/part/additional-paid-in-capital@1"),
        ),
        (
            BalanceSheetConcept::AccumulatedOtherComprehensiveIncome.element(),
            rule("us-gaap/alias/accumulated-other-comprehensive-income@1"),
        ),
        (
            DebtMaturityConcept::LongTermDebtIncludingCurrentMaturities.element(),
            rule("us-gaap/alias/long-term-debt-including-current-maturities@1"),
        ),
    ]);

    let result: BTreeSet<(CanonicalElementKind, RuleId)> =
        newest_filing(&candidates_of(&facts).candidates)
            .facts
            .into_iter()
            .map(|fact| (fact.concept, fact.rule))
            .collect();

    assert_eq!(result, expected_result);
}
