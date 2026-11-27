//! # Statement Real Filing Integration Tests
//!
//! Builds the three statements of one real filing, the 10-Q `0001628280-26-068246`, from the
//! standardized facts that the dedup rules give for its Notes facts and its instance facts.
//!
//! Each test writes out every line of one statement. The other lines close each open group, so
//! each statement adds up. A group whose facts have two exact starts gets no other line.
//!
//! ## External Dependencies
//!
//! None. The fixtures live in `src/tests/fixtures/data/`.

use bigdecimal::BigDecimal;
use jiff::civil::Date;
use pretty_assertions::assert_eq;
use xbrl::core::alias::resolve_aliases;
use xbrl::core::dedup::{more_precise, newest_filing};
use xbrl::core::elements::statement::balance_sheet::BalanceSheetGroup;
use xbrl::core::elements::statement::cash_flow::CashFlowGroup;
use xbrl::core::elements::statement::income_statement::IncomeStatementGroup;
use xbrl::core::elements::{
    BalanceSheetConcept, CanonicalElementKind, CashFlowConcept, GroupName, IncomeStatementConcept,
    PerShareConcept, StatementConcept,
};
use xbrl::core::fact::AsIsFact;
use xbrl::core::standardized::StandardizedFact;
use xbrl::core::statement::{InvalidStatementFacts, Line, Statement, StatementKind};
use xbrl::core::taxonomy::registry::{Plugin, Registry};
use xbrl::tests::fixtures::data::instance::parse_instance_fixture;
use xbrl::tests::fixtures::data::notes::parse_notes_fixture;
use xbrl::tests::fixtures::sample_fact::date;
use xbrl::us_gaap::UsGaap;

/// Runs the dedup rules and tiers 1 and 2 on the Notes facts of the fixture filing.
fn standardized_facts() -> Vec<StandardizedFact> {
    standardized(&parse_notes_fixture().facts)
}

/// Runs the dedup rules and tiers 1 and 2 on `facts`.
fn standardized(facts: &[AsIsFact]) -> Vec<StandardizedFact> {
    let plugin = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&plugin)])
        .expect("Given one plug-in with distinct families, the creation should always succeed");
    newest_filing(&resolve_aliases(&more_precise(facts), &registry).candidates).facts
}

/// Returns the facts without dimensions that end on `end`, last `quarters` quarters, and have a
/// concept that `keep` accepts.
fn facts_of(
    facts: &[StandardizedFact],
    end: Date,
    quarters: u16,
    keep: fn(CanonicalElementKind) -> bool,
) -> Vec<&StandardizedFact> {
    facts
        .iter()
        .filter(|fact| {
            fact.dimensions.is_empty()
                && fact.period.end() == end
                && fact.period.quarters() == quarters
                && keep(fact.concept)
        })
        .collect()
}

/// Returns the content and the value of each line.
fn lines(statement: &Statement) -> Vec<(Line, Option<BigDecimal>)> {
    statement
        .lines()
        .iter()
        .map(|line| (line.line(), line.value().cloned()))
        .collect()
}

/// Returns `millions` million.
fn millions(millions: i64) -> BigDecimal {
    BigDecimal::from(millions * 1_000_000)
}

fn standard(concept: impl Into<StatementConcept>) -> Line {
    Line::Standard(CanonicalElementKind::from(concept.into()))
}

const fn other(group: GroupName) -> Line {
    Line::Other { group }
}

#[test]
fn should_add_up_the_balance_sheet_of_2026_09_30_for_the_fixture_filing() {
    let facts = standardized_facts();
    let balance_sheet_facts = facts_of(&facts, date(2026, 9, 30), 0, |concept| {
        matches!(
            concept,
            CanonicalElementKind::Statement(StatementConcept::BalanceSheet(_))
        )
    });

    // Non-current assets and liabilities have no value, so their children count in `assets` and
    // `liabilities-and-equity`.
    let expected_result = Ok(vec![
        (standard(BalanceSheetConcept::Assets), Some(millions(7_216))),
        (
            standard(BalanceSheetConcept::CurrentAssets),
            Some(millions(1_846)),
        ),
        (
            standard(BalanceSheetConcept::CashAndEquivalents),
            Some(millions(388)),
        ),
        (standard(BalanceSheetConcept::ShortTermInvestments), None),
        (standard(BalanceSheetConcept::AccountsReceivable), None),
        (standard(BalanceSheetConcept::Inventory), None),
        (
            other(BalanceSheetGroup::CurrentAssets.name()),
            Some(millions(1_458)),
        ),
        (standard(BalanceSheetConcept::PropertyPlantEquipment), None),
        (standard(BalanceSheetConcept::Goodwill), Some(millions(602))),
        (standard(BalanceSheetConcept::IntangibleAssets), None),
        (
            other(BalanceSheetGroup::Assets.name()),
            Some(millions(4_768)),
        ),
        (
            standard(BalanceSheetConcept::LiabilitiesAndEquity),
            Some(millions(7_216)),
        ),
        (
            standard(BalanceSheetConcept::CurrentLiabilities),
            Some(millions(1_136)),
        ),
        (
            standard(BalanceSheetConcept::AccountsPayable),
            Some(millions(210)),
        ),
        (
            standard(BalanceSheetConcept::ShortTermDebt),
            Some(millions(45)),
        ),
        (
            other(BalanceSheetGroup::CurrentLiabilities.name()),
            Some(millions(881)),
        ),
        (
            standard(BalanceSheetConcept::NonCurrentLiabilities),
            Some(millions(6_332)),
        ),
        (standard(BalanceSheetConcept::LongTermDebt), None),
        (
            other(BalanceSheetGroup::NonCurrentLiabilities.name()),
            Some(millions(6_332)),
        ),
        (standard(BalanceSheetConcept::TemporaryEquity), None),
        (standard(BalanceSheetConcept::Equity), Some(millions(-252))),
        (
            standard(BalanceSheetConcept::EquityAttributableToParent),
            Some(millions(-252)),
        ),
        (
            standard(BalanceSheetConcept::AdditionalPaidInCapital),
            Some(millions(19)),
        ),
        (
            standard(BalanceSheetConcept::RetainedEarnings),
            Some(millions(-280)),
        ),
        (standard(BalanceSheetConcept::TreasuryStock), None),
        (
            standard(BalanceSheetConcept::AccumulatedOtherComprehensiveIncome),
            Some(millions(-6)),
        ),
        (
            other(BalanceSheetGroup::EquityAttributableToParent.name()),
            Some(millions(15)),
        ),
        (standard(BalanceSheetConcept::NoncontrollingInterest), None),
        (
            other(BalanceSheetGroup::LiabilitiesAndEquity.name()),
            Some(millions(0)),
        ),
    ]);

    let result = Statement::build(StatementKind::BalanceSheet, balance_sheet_facts)
        .map(|statement| lines(&statement));

    assert_eq!(result, expected_result);
}

#[test]
fn should_add_up_the_income_statement_of_june_2026_for_the_fixture_filing() {
    let facts = standardized_facts();
    // June lasts 30 days, so its facts have 0 quarters.
    let income_statement_facts = facts_of(&facts, date(2026, 6, 30), 0, |concept| {
        matches!(
            concept,
            CanonicalElementKind::Statement(StatementConcept::IncomeStatement(_))
        )
    });

    // Gross profit and operating expenses have no value, so operating income starts from
    // revenue: 130 - 839 = -709.
    let expected_result = Ok(vec![
        (
            standard(IncomeStatementConcept::Revenue),
            Some(millions(839)),
        ),
        (standard(IncomeStatementConcept::CostOfRevenue), None),
        (standard(IncomeStatementConcept::SellingGeneralAdmin), None),
        (standard(IncomeStatementConcept::ResearchDevelopment), None),
        (
            other(IncomeStatementGroup::OperatingIncome.name()),
            Some(millions(-709)),
        ),
        (
            standard(IncomeStatementConcept::OperatingIncome),
            Some(millions(130)),
        ),
        (standard(IncomeStatementConcept::InterestExpense), None),
        (
            other(IncomeStatementGroup::PretaxIncome.name()),
            Some(millions(-16)),
        ),
        (
            standard(IncomeStatementConcept::PretaxIncome),
            Some(millions(114)),
        ),
        (
            standard(IncomeStatementConcept::IncomeTaxExpense),
            Some(millions(31)),
        ),
        (
            other(IncomeStatementGroup::NetIncome.name()),
            Some(millions(0)),
        ),
        (
            standard(IncomeStatementConcept::NetIncome),
            Some(millions(83)),
        ),
        (
            standard(IncomeStatementConcept::NetIncomeAttributableToParent),
            Some(millions(83)),
        ),
        (
            standard(IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest),
            None,
        ),
        (
            standard(IncomeStatementConcept::OtherComprehensiveIncome),
            None,
        ),
    ]);

    let result = Statement::build(StatementKind::IncomeStatement, income_statement_facts)
        .map(|statement| lines(&statement));

    assert_eq!(result, expected_result);
}

/// Builds the cash flow statement of the quarter to 2026-09-30 from the standardized `facts`, with
/// net income as the start of `operating-cash-flow`.
fn cash_flow_of_the_quarter_to_2026_09_30(
    facts: &[StandardizedFact],
) -> Result<Vec<(Line, Option<BigDecimal>)>, InvalidStatementFacts> {
    let cash_flow_facts = facts_of(facts, date(2026, 9, 30), 1, |concept| {
        matches!(
            concept,
            CanonicalElementKind::Statement(StatementConcept::CashFlow(_))
        ) || concept == IncomeStatementConcept::NetIncome.element()
    });
    Statement::build(StatementKind::CashFlow, cash_flow_facts).map(|statement| lines(&statement))
}

/// The lines of the cash flow statement of the quarter to 2026-09-30, in both sources.
///
/// Net income and D&A cover the quarter from 1 July. The other lines cover four months from 1 June.
/// So `operating-cash-flow` mixes two starts and gets no other line. The other groups hold
/// four-month values only and keep their other lines.
fn expected_cash_flow_of_the_quarter_to_2026_09_30() -> Vec<(Line, Option<BigDecimal>)> {
    vec![
        (
            standard(CashFlowConcept::NetChangeInCash),
            Some(millions(137)),
        ),
        (
            standard(IncomeStatementConcept::NetIncome),
            Some(millions(147)),
        ),
        (
            standard(CashFlowConcept::DepreciationAmortization),
            Some(millions(114)),
        ),
        (
            standard(CashFlowConcept::ShareBasedCompensation),
            Some(millions(16)),
        ),
        (
            standard(CashFlowConcept::OperatingCashFlow),
            Some(millions(390)),
        ),
        (
            standard(CashFlowConcept::InvestingCashFlow),
            Some(millions(-138)),
        ),
        (
            standard(CashFlowConcept::CapitalExpenditures),
            Some(millions(151)),
        ),
        (
            other(CashFlowGroup::InvestingCashFlow.name()),
            Some(millions(13)),
        ),
        (
            standard(CashFlowConcept::FinancingCashFlow),
            Some(millions(-114)),
        ),
        (standard(CashFlowConcept::DividendsPaid), None),
        (standard(CashFlowConcept::ShareRepurchases), None),
        (
            other(CashFlowGroup::FinancingCashFlow.name()),
            Some(millions(-114)),
        ),
        (
            standard(CashFlowConcept::ExchangeRateEffect),
            Some(millions(-1)),
        ),
    ]
}

#[test]
fn should_build_the_cash_flow_statement_of_the_quarter_to_2026_09_30_from_the_notes_facts() {
    let facts = standardized_facts();

    let expected_result = Ok(expected_cash_flow_of_the_quarter_to_2026_09_30());

    let result = cash_flow_of_the_quarter_to_2026_09_30(&facts);

    assert_eq!(result, expected_result);
}

#[test]
fn should_build_the_cash_flow_statement_of_the_quarter_to_2026_09_30_from_the_instance_facts() {
    let facts = standardized(&parse_instance_fixture().facts);

    let expected_result = Ok(expected_cash_flow_of_the_quarter_to_2026_09_30());

    let result = cash_flow_of_the_quarter_to_2026_09_30(&facts);

    assert_eq!(result, expected_result);
}

#[test]
fn should_reject_the_per_share_facts_of_june_2026_when_building_the_income_statement() {
    let facts = standardized_facts();
    let june_facts = facts_of(&facts, date(2026, 6, 30), 0, |concept| {
        !matches!(
            concept,
            CanonicalElementKind::Statement(StatementConcept::CashFlow(_))
        )
    });

    let expected_result = Some(InvalidStatementFacts::UnexpectedConcept {
        kind: StatementKind::IncomeStatement,
        concept: PerShareConcept::EarningsPerShareBasic.element(),
    });

    let result = Statement::build(StatementKind::IncomeStatement, june_facts).err();

    assert_eq!(result, expected_result);
}
