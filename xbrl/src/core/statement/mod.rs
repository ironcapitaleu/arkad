//! # Financial Statements
//!
//! Provides [`Statement`], one financial statement of one entity and one period, built from
//! standardized facts.
//!
//! A statement lays out the lines of its roll-up tree and computes the other line of each open
//! group at read time. Silver stores no statement and no other line, so a statement always matches
//! the stored facts and the tree.

use std::collections::BTreeMap;
use std::fmt;

use bigdecimal::{BigDecimal, Zero};
use chrono::NaiveDate;
use thiserror::Error;

use super::elements::statement::{balance_sheet, cash_flow, income_statement};
use super::elements::{
    BalanceSheetConcept, CanonicalElementKind, CashFlowConcept, Group, GroupKind, GroupName,
    IncomeStatementConcept, Placed, Placement, Weight,
};
use super::filing::EntityId;
use super::period::Period;
use super::standardized::StandardizedFact;

/// The kind of a financial statement.
///
/// The kind fixes the concepts and the period type of the statement.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StatementKind {
    /// The balance sheet. Each value is an instant.
    BalanceSheet,
    /// The income statement, with comprehensive income at its end. Each value is a duration.
    IncomeStatement,
    /// The cash flow statement. Each value is a duration.
    CashFlow,
}

impl StatementKind {
    /// Returns `true` if the statement takes instants, and `false` if it takes durations.
    const fn takes_instants(self) -> bool {
        matches!(self, Self::BalanceSheet)
    }
}

impl fmt::Display for StatementKind {
    /// Writes the name for people, such as `balance sheet`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BalanceSheet => "balance sheet",
            Self::IncomeStatement => "income statement",
            Self::CashFlow => "cash flow statement",
        })
    }
}

/// The content of one line of a [`Statement`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Line {
    /// The line of a standard concept.
    Standard(CanonicalElementKind),
    /// The other line of a group: the parent less the start and the weighted children.
    Other {
        /// The group whose other line it is.
        group: GroupName,
    },
}

/// One line of a [`Statement`]: its content, its depth in the tree, its value, and its fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementLine {
    line: Line,
    depth: u8,
    value: Option<BigDecimal>,
    fact: Option<StandardizedFact>,
}

impl StatementLine {
    /// Returns the content of the line.
    #[must_use]
    pub const fn line(&self) -> Line {
        self.line
    }

    /// Returns the depth of the line in the tree. A top line has depth 0.
    #[must_use]
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    /// Returns the value, or `None` for a standard concept with no fact.
    #[must_use]
    pub const fn value(&self) -> Option<&BigDecimal> {
        self.value.as_ref()
    }

    /// Returns the standardized fact of a standard line, with its confidence, rule, and inputs.
    ///
    /// An other line and a standard line with no value have no fact.
    #[must_use]
    pub const fn fact(&self) -> Option<&StandardizedFact> {
        self.fact.as_ref()
    }
}

/// One financial statement of one entity and one period, laid out by its roll-up tree.
///
/// The fields are private, so a statement keeps the kind and the lines that
/// [`Statement::build`] gave it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    kind: StatementKind,
    entity: EntityId,
    period_end: NaiveDate,
    quarters: u16,
    lines: Vec<StatementLine>,
}

impl Statement {
    /// Builds the [`Statement`] of `kind` from the standardized facts of one entity and one period.
    ///
    /// The lines follow the tree of the statement. A breakdown shows its parent first, then its
    /// children one level deeper. A step shows its items one level deeper, then its parent. A step
    /// whose start belongs to another statement, such as `NetIncome` in the cash flow statement,
    /// shows the start first. A child with no value has a line with no value.
    ///
    /// An open group whose parent has a value ends with an other line: the parent less the start
    /// and the weighted children. A member with no value counts as zero. A closed group gets no
    /// other line, unless a transparent member opens it.
    ///
    /// A parent with no value is transparent: it has no line, its children keep their depth, and
    /// its members count in the group above it. If the start of a step has no value, the step uses
    /// the members of the step that makes the start, so it starts from the previous subtotal. A
    /// start with no value and no step that makes it counts as zero, so the other line holds it.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidStatementFacts`] if `facts` is empty, if a fact has no line in the
    /// statement, if a fact has the wrong period type, if two facts differ in entity, period,
    /// unit, or dimensions, or if two facts have the same concept. It returns the first error in
    /// the order of `facts`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::str::FromStr;
    ///
    /// use bigdecimal::BigDecimal;
    /// use xbrl::core::elements::BalanceSheetConcept;
    /// use xbrl::core::elements::statement::balance_sheet::BalanceSheetGroup;
    /// use xbrl::core::period::Period;
    /// use xbrl::core::statement::{Line, Statement, StatementKind};
    /// use xbrl::tests::fixtures::sample_fact::{date, standardized};
    ///
    /// let period = Period::Instant { end: date(2026, 9, 30) };
    /// let mut assets = standardized(period, "100");
    /// assets.concept = BalanceSheetConcept::Assets.element();
    /// let mut current_assets = standardized(period, "60");
    /// current_assets.concept = BalanceSheetConcept::CurrentAssets.element();
    /// let facts = [assets, current_assets];
    ///
    /// let expected_result = Some(BigDecimal::from_str("40").expect("Given a hardcoded decimal, the parse should always succeed"));
    ///
    /// let statement = Statement::build(StatementKind::BalanceSheet, &facts)
    ///     .expect("Given two instants of one entity, the build should always succeed");
    /// let other = Line::Other { group: BalanceSheetGroup::Assets.name() };
    /// let result = statement.lines().iter().find(|line| line.line() == other).and_then(|line| line.value()).cloned();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn build<'a>(
        kind: StatementKind,
        facts: impl IntoIterator<Item = &'a StandardizedFact>,
    ) -> Result<Self, InvalidStatementFacts> {
        let tree = Tree::of(kind);
        let facts = tree.check(facts)?;
        let Some(first) = facts.values().next() else {
            return Err(InvalidStatementFacts::MissingFacts { kind });
        };
        let (entity, period_end, quarters) = (
            first.entity.clone(),
            first.period.end(),
            first.period.quarters(),
        );
        let mut layout = Layout {
            tree: &tree,
            facts: &facts,
            lines: Vec::new(),
        };
        for (concept, placement) in &tree.placements {
            if *placement == Placement::Top {
                layout.concept(*concept, 0);
            }
        }
        Ok(Self {
            kind,
            entity,
            period_end,
            quarters,
            lines: layout.lines,
        })
    }

    /// Returns the kind of the statement.
    #[must_use]
    pub const fn kind(&self) -> StatementKind {
        self.kind
    }

    /// Returns the entity the statement is about.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Returns the end of the period: the date of a balance sheet, or the last day of a duration.
    #[must_use]
    pub const fn period_end(&self) -> NaiveDate {
        self.period_end
    }

    /// Returns the length of the period in quarters. A balance sheet has 0, and so does a duration
    /// shorter than half a quarter.
    #[must_use]
    pub const fn quarters(&self) -> u16 {
        self.quarters
    }

    /// Returns the lines in the order of the tree.
    #[must_use]
    pub fn lines(&self) -> &[StatementLine] {
        &self.lines
    }
}

/// Error representing a set of standardized facts that builds no [`Statement`].
///
/// [`Statement::build`] returns it before it lays out a line.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InvalidStatementFacts {
    /// The input holds no fact, so the statement has no entity and no period.
    #[error("[MissingFacts] No standardized fact to build the {kind} from")]
    MissingFacts {
        /// The kind of the statement.
        kind: StatementKind,
    },
    /// A fact has a concept that is neither a concept of the statement nor the start of one of
    /// its steps.
    #[error("[UnexpectedConcept] Concept '{}' has no line in the {kind}", .concept.name())]
    UnexpectedConcept {
        /// The kind of the statement.
        kind: StatementKind,
        /// The concept of the fact.
        concept: CanonicalElementKind,
    },
    /// A fact covers a duration in a balance sheet, or an instant in another statement.
    #[error("[UnexpectedPeriodType] Fact of '{}' covers '{period}', which the {kind} does not take", .concept.name())]
    UnexpectedPeriodType {
        /// The kind of the statement.
        kind: StatementKind,
        /// The concept of the fact.
        concept: CanonicalElementKind,
        /// The period of the fact.
        period: Period,
    },
    /// A fact differs from the first fact in entity, period, unit, or dimensions.
    #[error("[MismatchedFact] Fact of '{}' differs from the fact of '{}' in entity, period, unit, or dimensions", .concept.name(), .first.name())]
    MismatchedFact {
        /// The concept of the fact that differs.
        concept: CanonicalElementKind,
        /// The concept of the first fact.
        first: CanonicalElementKind,
    },
    /// Two facts have the same concept.
    #[error("[DuplicateConcept] Two facts of '{}'", .concept.name())]
    DuplicateConcept {
        /// The concept of both facts.
        concept: CanonicalElementKind,
    },
}

/// The roll-up tree of one statement: the placements of its concepts and its groups.
struct Tree {
    kind: StatementKind,
    placements: Vec<(CanonicalElementKind, Placement)>,
    groups: &'static [Group],
}

impl Tree {
    /// Returns the tree of `kind`, with the concepts in the order of their enum.
    fn of(kind: StatementKind) -> Self {
        fn placements<C: Placed>(
            element: fn(C) -> CanonicalElementKind,
        ) -> Vec<(CanonicalElementKind, Placement)> {
            C::ALL
                .iter()
                .map(|concept| (element(*concept), concept.placement()))
                .collect()
        }
        let (placements, groups) = match kind {
            StatementKind::BalanceSheet => (
                placements(BalanceSheetConcept::element),
                balance_sheet::GROUPS,
            ),
            StatementKind::IncomeStatement => (
                placements(IncomeStatementConcept::element),
                income_statement::GROUPS,
            ),
            StatementKind::CashFlow => (placements(CashFlowConcept::element), cash_flow::GROUPS),
        };
        Self {
            kind,
            placements,
            groups,
        }
    }

    /// Returns `true` if `concept` belongs to the statement.
    fn places(&self, concept: CanonicalElementKind) -> bool {
        self.placements.iter().any(|(placed, _)| *placed == concept)
    }

    /// Returns `true` if `concept` belongs to the statement or starts one of its steps.
    fn holds(&self, concept: CanonicalElementKind) -> bool {
        self.places(concept)
            || self
                .groups
                .iter()
                .any(|group| group.kind == GroupKind::Step { start: concept })
    }

    /// Checks `facts` against the statement, and returns them by concept.
    fn check<'a>(
        &self,
        facts: impl IntoIterator<Item = &'a StandardizedFact>,
    ) -> Result<BTreeMap<CanonicalElementKind, &'a StandardizedFact>, InvalidStatementFacts> {
        let kind = self.kind;
        let mut checked: BTreeMap<CanonicalElementKind, &StandardizedFact> = BTreeMap::new();
        let mut first: Option<&StandardizedFact> = None;
        for fact in facts {
            let concept = fact.concept;
            if !self.holds(concept) {
                return Err(InvalidStatementFacts::UnexpectedConcept { kind, concept });
            }
            if matches!(fact.period, Period::Instant { .. }) != kind.takes_instants() {
                let period = fact.period;
                return Err(InvalidStatementFacts::UnexpectedPeriodType {
                    kind,
                    concept,
                    period,
                });
            }
            let first = *first.get_or_insert(fact);
            if (&fact.entity, fact.period, &fact.unit, &fact.dimensions)
                != (&first.entity, first.period, &first.unit, &first.dimensions)
            {
                let first = first.concept;
                return Err(InvalidStatementFacts::MismatchedFact { concept, first });
            }
            if checked.insert(concept, fact).is_some() {
                return Err(InvalidStatementFacts::DuplicateConcept { concept });
            }
        }
        Ok(checked)
    }

    /// Returns the groups whose parent is `parent`, in the order of the table.
    fn groups_of(&self, parent: CanonicalElementKind) -> impl Iterator<Item = Group> {
        self.groups
            .iter()
            .filter(move |group| group.parent == parent)
            .copied()
    }

    /// Returns the group that makes `parent` when it has no value: its step, or else its first
    /// group.
    fn source_of(&self, parent: CanonicalElementKind) -> Option<Group> {
        self.groups_of(parent)
            .find(|group| matches!(group.kind, GroupKind::Step { .. }))
            .or_else(|| self.groups_of(parent).next())
    }

    /// Returns the children of `group` with their weights, in the order of their enum.
    fn children(
        &self,
        group: GroupName,
    ) -> impl Iterator<Item = (CanonicalElementKind, Weight)> + '_ {
        self.placements
            .iter()
            .filter_map(move |(concept, placement)| match placement {
                Placement::Child { group: of, weight } if *of == group => Some((*concept, *weight)),
                _ => None,
            })
    }

    /// Returns the members of `group` with their weights: the start of a step, then the children.
    fn members(&self, group: Group) -> Vec<(CanonicalElementKind, Weight)> {
        let start = match group.kind {
            GroupKind::Step { start } => Some((start, Weight::Plus)),
            GroupKind::Breakdown => None,
        };
        start.into_iter().chain(self.children(group.name)).collect()
    }
}

/// The lines of a statement while [`Statement::build`] lays them out.
struct Layout<'a> {
    tree: &'a Tree,
    facts: &'a BTreeMap<CanonicalElementKind, &'a StandardizedFact>,
    lines: Vec<StatementLine>,
}

impl Layout<'_> {
    /// Lays out `concept` at `depth`, with the groups whose parent it is.
    fn concept(&mut self, concept: CanonicalElementKind, depth: u8) {
        let groups: Vec<Group> = self.tree.groups_of(concept).collect();
        for group in groups.iter().copied() {
            if let GroupKind::Step { start } = group.kind {
                if !self.tree.places(start) {
                    self.standard(start, depth);
                }
                self.group(group, depth + 1);
            }
        }
        if groups.is_empty() || self.facts.contains_key(&concept) {
            self.standard(concept, depth);
        }
        for group in groups.iter().copied() {
            if group.kind == GroupKind::Breakdown {
                self.group(group, depth + 1);
            }
        }
    }

    /// Lays out the children of `group` at `depth`, then its other line.
    fn group(&mut self, group: Group, depth: u8) {
        for (child, _) in self.tree.children(group.name) {
            self.concept(child, depth);
        }
        if let Some(value) = self.other(group) {
            self.lines.push(StatementLine {
                line: Line::Other { group: group.name },
                depth,
                value: Some(value),
                fact: None,
            });
        }
    }

    /// Lays out the line of the standard concept `concept` at `depth`.
    fn standard(&mut self, concept: CanonicalElementKind, depth: u8) {
        let fact = self.facts.get(&concept).map(|fact| (*fact).clone());
        self.lines.push(StatementLine {
            line: Line::Standard(concept),
            depth,
            value: fact.as_ref().map(|fact| fact.value.clone()),
            fact,
        });
    }

    /// Returns the value of the other line of `group`, or `None` if the group has none.
    fn other(&self, group: Group) -> Option<BigDecimal> {
        let parent = &self.facts.get(&group.parent)?.value;
        let mut sum = BigDecimal::zero();
        let mut opened = false;
        for (member, weight) in self.tree.members(group) {
            self.add(member, weight, &mut sum, &mut opened);
        }
        (!group.closed || opened).then(|| parent - sum)
    }

    /// Adds `member` with `weight` to `sum`. A transparent member adds its own members, and sets
    /// `opened`.
    fn add(
        &self,
        member: CanonicalElementKind,
        weight: Weight,
        sum: &mut BigDecimal,
        opened: &mut bool,
    ) {
        match (self.facts.get(&member), self.tree.source_of(member)) {
            (Some(fact), _) => match weight {
                Weight::Plus => *sum += &fact.value,
                Weight::Minus => *sum -= &fact.value,
            },
            (None, Some(group)) => {
                *opened = true;
                for (inner, inner_weight) in self.tree.members(group) {
                    let combined = if inner_weight == weight {
                        Weight::Plus
                    } else {
                        Weight::Minus
                    };
                    self.add(inner, combined, sum, opened);
                }
            }
            (None, None) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::StatementConcept;
    use crate::core::elements::statement::balance_sheet::BalanceSheetGroup;
    use crate::core::elements::statement::cash_flow::CashFlowGroup;
    use crate::core::elements::statement::income_statement::IncomeStatementGroup;
    use crate::core::fact::Dimensions;
    use crate::core::period::Duration;
    use crate::core::unit::Unit;
    use crate::tests::fixtures::sample_fact::{date, standardized};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_statement_kind() {
        implements_auto_traits::<StatementKind>();
    }

    #[test]
    const fn should_implement_auto_traits_for_line() {
        implements_auto_traits::<Line>();
    }

    #[test]
    const fn should_implement_auto_traits_for_statement() {
        implements_auto_traits::<Statement>();
    }

    #[test]
    const fn should_implement_auto_traits_for_statement_line() {
        implements_auto_traits::<StatementLine>();
    }

    #[test]
    const fn should_implement_auto_traits_for_invalid_statement_facts() {
        implements_auto_traits::<InvalidStatementFacts>();
    }

    fn number(text: &str) -> BigDecimal {
        BigDecimal::from_str(text)
            .expect("Given a hardcoded decimal text, the parse should always succeed")
    }

    /// Returns the period of the tests: the instant 2026-09-30 for a balance sheet concept, and
    /// the quarter that ends on it for every other concept.
    fn period_of(concept: CanonicalElementKind) -> Period {
        let end = date(2026, 9, 30);
        match concept {
            CanonicalElementKind::Statement(StatementConcept::BalanceSheet(_)) => {
                Period::Instant { end }
            }
            _ => Period::Duration(Duration::from_end_and_quarters(end, 1)),
        }
    }

    /// Returns the standardized fact `concept` = `value` of Apple in USD for [`period_of`].
    fn fact(concept: impl Into<StatementConcept>, value: &str) -> StandardizedFact {
        let concept = CanonicalElementKind::from(concept.into());
        let mut fact = standardized(period_of(concept), value);
        fact.concept = concept;
        fact
    }

    /// Returns the content and the depth of each line.
    fn layout(statement: &Statement) -> Vec<(Line, u8)> {
        statement
            .lines()
            .iter()
            .map(|line| (line.line(), line.depth()))
            .collect()
    }

    /// Returns the group and the value of each other line.
    fn others(statement: &Statement) -> Vec<(GroupName, Option<BigDecimal>)> {
        statement
            .lines()
            .iter()
            .filter_map(|line| match line.line() {
                Line::Other { group } => Some((group, line.value().cloned())),
                Line::Standard(_) => None,
            })
            .collect()
    }

    fn standard(concept: impl Into<StatementConcept>) -> Line {
        Line::Standard(CanonicalElementKind::from(concept.into()))
    }

    fn other(group: GroupName) -> Line {
        Line::Other { group }
    }

    #[test]
    fn should_lay_out_breakdowns_parent_first_and_steps_parent_last_for_the_cash_flow_statement() {
        let facts = [
            fact(CashFlowConcept::NetChangeInCash, "137"),
            fact(CashFlowConcept::OperatingCashFlow, "390"),
            fact(CashFlowConcept::InvestingCashFlow, "-138"),
            fact(CashFlowConcept::FinancingCashFlow, "-114"),
        ];

        let expected_result = Ok(vec![
            (standard(CashFlowConcept::NetChangeInCash), 0),
            (standard(IncomeStatementConcept::NetIncome), 1),
            (standard(CashFlowConcept::DepreciationAmortization), 2),
            (standard(CashFlowConcept::ShareBasedCompensation), 2),
            (other(CashFlowGroup::OperatingCashFlow.name()), 2),
            (standard(CashFlowConcept::OperatingCashFlow), 1),
            (standard(CashFlowConcept::InvestingCashFlow), 1),
            (standard(CashFlowConcept::CapitalExpenditures), 2),
            (other(CashFlowGroup::InvestingCashFlow.name()), 2),
            (standard(CashFlowConcept::FinancingCashFlow), 1),
            (standard(CashFlowConcept::DividendsPaid), 2),
            (standard(CashFlowConcept::ShareRepurchases), 2),
            (other(CashFlowGroup::FinancingCashFlow.name()), 2),
            (standard(CashFlowConcept::ExchangeRateEffect), 1),
        ]);

        let result = Statement::build(StatementKind::CashFlow, &facts).map(|s| layout(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_lay_out_the_cascade_from_revenue_to_comprehensive_income_for_the_income_statement() {
        let facts = [
            fact(IncomeStatementConcept::Revenue, "100"),
            fact(IncomeStatementConcept::GrossProfit, "40"),
            fact(IncomeStatementConcept::OperatingExpenses, "25"),
            fact(IncomeStatementConcept::OperatingIncome, "15"),
            fact(IncomeStatementConcept::PretaxIncome, "12"),
            fact(IncomeStatementConcept::NetIncome, "9"),
            fact(IncomeStatementConcept::ComprehensiveIncome, "10"),
        ];

        let expected_result = Ok(vec![
            (standard(IncomeStatementConcept::Revenue), 0),
            (standard(IncomeStatementConcept::CostOfRevenue), 1),
            (standard(IncomeStatementConcept::GrossProfit), 0),
            (standard(IncomeStatementConcept::OperatingExpenses), 1),
            (standard(IncomeStatementConcept::SellingGeneralAdmin), 2),
            (standard(IncomeStatementConcept::ResearchDevelopment), 2),
            (other(IncomeStatementGroup::OperatingExpenses.name()), 2),
            (other(IncomeStatementGroup::OperatingIncome.name()), 1),
            (standard(IncomeStatementConcept::OperatingIncome), 0),
            (standard(IncomeStatementConcept::InterestExpense), 1),
            (other(IncomeStatementGroup::PretaxIncome.name()), 1),
            (standard(IncomeStatementConcept::PretaxIncome), 0),
            (standard(IncomeStatementConcept::IncomeTaxExpense), 1),
            (other(IncomeStatementGroup::NetIncome.name()), 1),
            (standard(IncomeStatementConcept::NetIncome), 0),
            (
                standard(IncomeStatementConcept::NetIncomeAttributableToParent),
                1,
            ),
            (
                standard(IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest),
                1,
            ),
            (
                standard(IncomeStatementConcept::OtherComprehensiveIncome),
                1,
            ),
            (standard(IncomeStatementConcept::ComprehensiveIncome), 0),
        ]);

        let result = Statement::build(StatementKind::IncomeStatement, &facts).map(|s| layout(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_a_child_with_no_value_as_zero_when_computing_the_other_line_of_an_open_group() {
        let facts = [
            fact(BalanceSheetConcept::CurrentAssets, "100"),
            fact(BalanceSheetConcept::CashAndEquivalents, "30"),
        ];

        let expected_result = Ok(vec![(
            BalanceSheetGroup::CurrentAssets.name(),
            Some(number("70")),
        )]);

        let result = Statement::build(StatementKind::BalanceSheet, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_a_child_of_weight_minus_when_computing_the_other_line() {
        let facts = [
            fact(BalanceSheetConcept::EquityAttributableToParent, "100"),
            fact(BalanceSheetConcept::RetainedEarnings, "80"),
            fact(BalanceSheetConcept::TreasuryStock, "10"),
        ];

        let expected_result = Ok(vec![(
            BalanceSheetGroup::EquityAttributableToParent.name(),
            Some(number("30")),
        )]);

        let result = Statement::build(StatementKind::BalanceSheet, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_other_line_when_the_parent_of_an_open_group_has_no_value() {
        let facts = [fact(BalanceSheetConcept::CashAndEquivalents, "30")];

        let expected_result = Ok(Vec::new());

        let result = Statement::build(StatementKind::BalanceSheet, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_the_children_in_the_group_above_when_their_parent_has_no_value() {
        // A bank reports no current subtotals, so cash and goodwill count against assets.
        let facts = [
            fact(BalanceSheetConcept::Assets, "100"),
            fact(BalanceSheetConcept::CashAndEquivalents, "30"),
            fact(BalanceSheetConcept::Goodwill, "20"),
        ];

        let expected_result = Ok(vec![(BalanceSheetGroup::Assets.name(), Some(number("50")))]);

        let result = Statement::build(StatementKind::BalanceSheet, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_line_to_a_parent_with_no_value() {
        let facts = [
            fact(BalanceSheetConcept::Assets, "100"),
            fact(BalanceSheetConcept::CashAndEquivalents, "30"),
        ];

        let expected_result = Ok(vec![
            (standard(BalanceSheetConcept::Assets), 0),
            (standard(BalanceSheetConcept::CashAndEquivalents), 2),
            (standard(BalanceSheetConcept::ShortTermInvestments), 2),
            (standard(BalanceSheetConcept::AccountsReceivable), 2),
            (standard(BalanceSheetConcept::Inventory), 2),
            (standard(BalanceSheetConcept::PropertyPlantEquipment), 2),
            (standard(BalanceSheetConcept::Goodwill), 2),
            (standard(BalanceSheetConcept::IntangibleAssets), 2),
            (other(BalanceSheetGroup::Assets.name()), 1),
            (standard(BalanceSheetConcept::AccountsPayable), 3),
            (standard(BalanceSheetConcept::ShortTermDebt), 3),
            (standard(BalanceSheetConcept::LongTermDebt), 3),
            (standard(BalanceSheetConcept::TemporaryEquity), 1),
            (standard(BalanceSheetConcept::AdditionalPaidInCapital), 3),
            (standard(BalanceSheetConcept::RetainedEarnings), 3),
            (standard(BalanceSheetConcept::TreasuryStock), 3),
            (
                standard(BalanceSheetConcept::AccumulatedOtherComprehensiveIncome),
                3,
            ),
            (standard(BalanceSheetConcept::NoncontrollingInterest), 2),
        ]);

        let result = Statement::build(StatementKind::BalanceSheet, &facts).map(|s| layout(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_add_the_items_of_a_transparent_breakdown_with_the_weight_of_its_parent() {
        // Operating expenses has no value, so its items count with the weight -1 of operating
        // expenses: 40 - 100 + 30 + 20 = -10.
        let facts = [
            fact(IncomeStatementConcept::GrossProfit, "100"),
            fact(IncomeStatementConcept::SellingGeneralAdmin, "30"),
            fact(IncomeStatementConcept::ResearchDevelopment, "20"),
            fact(IncomeStatementConcept::OperatingIncome, "40"),
        ];

        let expected_result = Ok(vec![(
            IncomeStatementGroup::OperatingIncome.name(),
            Some(number("-10")),
        )]);

        let result = Statement::build(StatementKind::IncomeStatement, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_start_the_step_from_the_previous_subtotal_when_the_start_has_no_value() {
        // A single-step filer reports no gross profit, so operating income starts from revenue,
        // and the cost of revenue counts in the step: 25 - 100 + 60 = -15.
        let facts = [
            fact(IncomeStatementConcept::Revenue, "100"),
            fact(IncomeStatementConcept::CostOfRevenue, "60"),
            fact(IncomeStatementConcept::OperatingIncome, "25"),
        ];

        let expected_result = Ok(vec![(
            IncomeStatementGroup::OperatingIncome.name(),
            Some(number("-15")),
        )]);

        let result = Statement::build(StatementKind::IncomeStatement, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_start_the_step_from_zero_when_the_start_has_no_value_and_no_previous_subtotal() {
        // No net income, so the other line holds it: 390 - 16 = 374.
        let facts = [
            fact(CashFlowConcept::OperatingCashFlow, "390"),
            fact(CashFlowConcept::ShareBasedCompensation, "16"),
        ];

        let expected_result = Ok(vec![(
            CashFlowGroup::OperatingCashFlow.name(),
            Some(number("374")),
        )]);

        let result = Statement::build(StatementKind::CashFlow, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_the_start_from_another_statement_when_it_has_a_value() {
        let facts = [
            fact(IncomeStatementConcept::NetIncome, "230"),
            fact(CashFlowConcept::OperatingCashFlow, "390"),
            fact(CashFlowConcept::DepreciationAmortization, "152"),
            fact(CashFlowConcept::ShareBasedCompensation, "16"),
        ];

        let expected_result = Ok(vec![(
            CashFlowGroup::OperatingCashFlow.name(),
            Some(number("-8")),
        )]);

        let result = Statement::build(StatementKind::CashFlow, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_no_other_line_to_a_closed_group_when_a_member_has_no_value() {
        // The noncontrolling interest has no value, but it is no parent, so `equity` stays closed.
        let facts = [
            fact(BalanceSheetConcept::Equity, "100"),
            fact(BalanceSheetConcept::EquityAttributableToParent, "90"),
        ];

        let expected_result = Ok(vec![(
            BalanceSheetGroup::EquityAttributableToParent.name(),
            Some(number("90")),
        )]);

        let result = Statement::build(StatementKind::BalanceSheet, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_an_other_line_to_a_closed_group_when_a_transparent_member_opens_it() {
        // Liabilities has no value, so its children count in `liabilities-and-equity`:
        // 100 - 30 - 50 - 15 = 5. Equity attributable to parent has no value either, so `equity`
        // opens too.
        let facts = [
            fact(BalanceSheetConcept::LiabilitiesAndEquity, "100"),
            fact(BalanceSheetConcept::CurrentLiabilities, "30"),
            fact(BalanceSheetConcept::NonCurrentLiabilities, "50"),
            fact(BalanceSheetConcept::Equity, "15"),
        ];

        let expected_result = Ok(vec![
            (
                BalanceSheetGroup::CurrentLiabilities.name(),
                Some(number("30")),
            ),
            (
                BalanceSheetGroup::NonCurrentLiabilities.name(),
                Some(number("50")),
            ),
            (BalanceSheetGroup::Equity.name(), Some(number("15"))),
            (
                BalanceSheetGroup::LiabilitiesAndEquity.name(),
                Some(number("5")),
            ),
        ]);

        let result = Statement::build(StatementKind::BalanceSheet, &facts).map(|s| others(&s));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_standard_line_its_fact_when_the_concept_has_a_value() {
        let goodwill = fact(BalanceSheetConcept::Goodwill, "602");
        let facts = [goodwill.clone()];

        let expected_result = Ok(Some(goodwill));

        let result = Statement::build(StatementKind::BalanceSheet, &facts).map(|statement| {
            statement
                .lines()
                .iter()
                .find(|line| line.line() == standard(BalanceSheetConcept::Goodwill))
                .and_then(StatementLine::fact)
                .cloned()
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_kind_and_the_period_of_the_facts_for_the_cash_flow_statement() {
        let facts = [fact(CashFlowConcept::OperatingCashFlow, "390")];

        let expected_result = Ok((StatementKind::CashFlow, date(2026, 9, 30), 1));

        let result = Statement::build(StatementKind::CashFlow, &facts)
            .map(|s| (s.kind(), s.period_end(), s.quarters()));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_facts_when_there_is_none() {
        let facts: [StandardizedFact; 0] = [];

        let expected_result = Some(InvalidStatementFacts::MissingFacts {
            kind: StatementKind::BalanceSheet,
        });

        let result = Statement::build(StatementKind::BalanceSheet, &facts).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_fact_of_another_statement_when_it_starts_no_step() {
        let revenue = fact(IncomeStatementConcept::Revenue, "100");
        let facts = [revenue];

        let expected_result = Some(InvalidStatementFacts::UnexpectedConcept {
            kind: StatementKind::CashFlow,
            concept: IncomeStatementConcept::Revenue.element(),
        });

        let result = Statement::build(StatementKind::CashFlow, &facts).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_duration_when_building_a_balance_sheet() {
        let mut assets = fact(BalanceSheetConcept::Assets, "100");
        let quarter = Period::Duration(Duration::from_end_and_quarters(date(2026, 9, 30), 1));
        assets.period = quarter;
        let facts = [assets];

        let expected_result = Some(InvalidStatementFacts::UnexpectedPeriodType {
            kind: StatementKind::BalanceSheet,
            concept: BalanceSheetConcept::Assets.element(),
            period: quarter,
        });

        let result = Statement::build(StatementKind::BalanceSheet, &facts).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_fact_when_its_period_differs_from_the_first_fact() {
        let mut goodwill = fact(BalanceSheetConcept::Goodwill, "602");
        goodwill.period = Period::Instant {
            end: date(2026, 5, 31),
        };
        let facts = [fact(BalanceSheetConcept::Assets, "100"), goodwill];

        let expected_result = Some(InvalidStatementFacts::MismatchedFact {
            concept: BalanceSheetConcept::Goodwill.element(),
            first: BalanceSheetConcept::Assets.element(),
        });

        let result = Statement::build(StatementKind::BalanceSheet, &facts).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_facts_when_two_have_the_same_concept() {
        let facts = [
            fact(BalanceSheetConcept::Assets, "100"),
            fact(BalanceSheetConcept::Assets, "100"),
        ];

        let expected_result = Some(InvalidStatementFacts::DuplicateConcept {
            concept: BalanceSheetConcept::Assets.element(),
        });

        let result = Statement::build(StatementKind::BalanceSheet, &facts).err();

        assert_eq!(result, expected_result);
    }

    /// Returns a balance sheet of assets, and a goodwill fact that `change` makes differ from it.
    fn assets_and_goodwill(change: fn(&mut StandardizedFact)) -> [StandardizedFact; 2] {
        let mut goodwill = fact(BalanceSheetConcept::Goodwill, "602");
        change(&mut goodwill);
        [fact(BalanceSheetConcept::Assets, "100"), goodwill]
    }

    #[test]
    fn should_reject_a_fact_when_its_entity_differs_from_the_first_fact() {
        let facts = assets_and_goodwill(|goodwill| {
            goodwill.entity = EntityId::new("cik", "0000789019")
                .expect("Given a hardcoded scheme and CIK, the creation should always succeed");
        });

        let expected_result = Some(InvalidStatementFacts::MismatchedFact {
            concept: BalanceSheetConcept::Goodwill.element(),
            first: BalanceSheetConcept::Assets.element(),
        });

        let result = Statement::build(StatementKind::BalanceSheet, &facts).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_fact_when_its_unit_differs_from_the_first_fact() {
        let facts = assets_and_goodwill(|goodwill| {
            goodwill.unit = Unit::from_str("EUR")
                .expect("Given a hardcoded unit with one measure, the parse should always succeed");
        });

        let expected_result = Some(InvalidStatementFacts::MismatchedFact {
            concept: BalanceSheetConcept::Goodwill.element(),
            first: BalanceSheetConcept::Assets.element(),
        });

        let result = Statement::build(StatementKind::BalanceSheet, &facts).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_a_fact_when_its_dimensions_differ_from_the_first_fact() {
        let facts = assets_and_goodwill(|goodwill| {
            goodwill.dimensions =
                Dimensions::new([("StatementBusinessSegmentsAxis", "FreightMember")]).expect(
                    "Given one hardcoded axis and member, the creation should always succeed",
                );
        });

        let expected_result = Some(InvalidStatementFacts::MismatchedFact {
            concept: BalanceSheetConcept::Goodwill.element(),
            first: BalanceSheetConcept::Assets.element(),
        });

        let result = Statement::build(StatementKind::BalanceSheet, &facts).err();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_statement_when_displaying_missing_facts() {
        let error = InvalidStatementFacts::MissingFacts {
            kind: StatementKind::BalanceSheet,
        };

        let expected_result = "[MissingFacts] No standardized fact to build the balance sheet from";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_concept_and_the_period_when_displaying_an_unexpected_period_type() {
        let error = InvalidStatementFacts::UnexpectedPeriodType {
            kind: StatementKind::IncomeStatement,
            concept: IncomeStatementConcept::Revenue.element(),
            period: Period::Instant {
                end: date(2026, 9, 30),
            },
        };

        let expected_result = "[UnexpectedPeriodType] Fact of 'Revenue' covers '2026-09-30', which the income statement does not take";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_both_concepts_when_displaying_a_mismatched_fact() {
        let error = InvalidStatementFacts::MismatchedFact {
            concept: BalanceSheetConcept::Goodwill.element(),
            first: BalanceSheetConcept::Assets.element(),
        };

        let expected_result = "[MismatchedFact] Fact of 'Goodwill' differs from the fact of 'Assets' in entity, period, unit, or dimensions";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_concept_when_displaying_a_duplicate_concept() {
        let error = InvalidStatementFacts::DuplicateConcept {
            concept: BalanceSheetConcept::Assets.element(),
        };

        let expected_result = "[DuplicateConcept] Two facts of 'Assets'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_name_the_concept_and_the_statement_when_displaying_an_unexpected_concept() {
        let error = InvalidStatementFacts::UnexpectedConcept {
            kind: StatementKind::CashFlow,
            concept: IncomeStatementConcept::Revenue.element(),
        };

        let expected_result =
            "[UnexpectedConcept] Concept 'Revenue' has no line in the cash flow statement";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
