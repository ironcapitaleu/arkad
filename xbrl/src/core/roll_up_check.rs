//! # Roll-Up Check
//!
//! Provides [`check_roll_ups`], the checks of the roll-up tree on the standardized series of one
//! entity. Each failed check is a [`Finding`]. A check never removes a value.

use std::collections::{BTreeMap, BTreeSet};

use bigdecimal::{BigDecimal, Signed};
use chrono::NaiveDate;

use super::check_tolerance::{exceeds_tolerance, weakest};
use super::closed_group::{members, roll_up_rule, signed};
use super::dedup::Series;
use super::elements::statement::balance_sheet::BalanceSheetGroup;
use super::elements::statement::groups;
use super::elements::statement::income_statement::IncomeStatementGroup;
use super::elements::{
    BalanceSheetConcept, CanonicalElementKind, Group, GroupName, StatementConcept,
};
use super::fact::Dimensions;
use super::filing::EntityId;
use super::finding::{Finding, RollUpCheck};
use super::period::Period;
use super::standardized::StandardizedFact;
use super::statement::{Line, Statement, StatementKind, mixed_starts};
use super::taxonomy::reference::PeriodType;
use super::unit::Unit;

/// The groups whose other line must not fall below zero.
///
/// Each child of these groups is a part of the parent with the same sign. Other lines of steps and
/// of equity can fall below zero, so they get no check.
const NEGATIVE_OTHER_GROUPS: [GroupName; 5] = [
    BalanceSheetGroup::CurrentAssets.name(),
    BalanceSheetGroup::NonCurrentAssets.name(),
    BalanceSheetGroup::CurrentLiabilities.name(),
    BalanceSheetGroup::NonCurrentLiabilities.name(),
    IncomeStatementGroup::OperatingExpenses.name(),
];

/// The tolerance of a check in basis points of its base: 10, or 0.1%.
const TOLERANCE_BASIS_POINTS: u32 = 10;

/// Runs the checks of the roll-up tree on the series of one entity, and returns a finding for each
/// failed check.
///
/// `added` holds the facts that later rules give for the series, such as
/// `core/period/q4-from-fy@1` and `core/roll-up/<group>@1`. The checks read them as they read
/// `series.facts`. The checks read only values without dimensions. They run on each period and
/// unit, in this order:
///
/// 1. Each closed group, in the order of [`groups`]: the start and the weighted children must
///    equal the parent. A failure is a [`Finding::RollUpMismatch`].
/// 2. The balance sheet identity: `Assets` must equal `LiabilitiesAndEquity`. A failure is a
///    [`Finding::RollUpMismatch`].
/// 3. The other line of `current-assets`, `non-current-assets`, `current-liabilities`,
///    `non-current-liabilities`, and `operating-expenses` must not fall below zero. A failure is a
///    [`Finding::NegativeOther`]. The other line is the one of [`Statement`].
///
/// A check passes within ±0.1% of the largest absolute value of its values, not of the parent. In
/// a step, the parent can be near zero while its members are large.
///
/// A closed group gets no check for a period if a member has no value and is not zero if absent,
/// or if dedup blocks a member. A child that is zero if absent counts as zero,
/// `ExchangeRateEffect` included. A closed
/// group also gets no check for a period if `core/roll-up/<group>@1` derived a value of that group
/// for the period, because the group then holds by construction.
///
/// Before these checks, each group of the slot, open or closed, gets a [`Finding::MixedStarts`] if
/// two of the facts that [`Statement`] counts in it have different exact starts. Such a group gets
/// no other check and no other line. The other groups of the slot keep their checks. A value
/// without an exact start mixes with no other.
#[must_use]
pub fn check_roll_ups(series: &Series, added: &[StandardizedFact]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for slot in Slot::all(series, added) {
        findings.extend(groups().filter_map(|group| slot.check_starts(*group)));
        findings.extend(
            groups()
                .filter(|group| group.closed)
                .filter_map(|group| slot.check_closed_group(*group)),
        );
        findings.extend(slot.check_balance_sheet());
        findings.extend(slot.check_negative_others());
    }
    findings
}

/// The key of a [`Slot`]: the entity, the period end, the length in quarters, the period type, the
/// unit, and the dimensions, which are always empty.
type SlotKey<'a> = (
    &'a EntityId,
    NaiveDate,
    u16,
    PeriodType,
    &'a Unit,
    &'a Dimensions,
);

/// The values without dimensions of one entity, period, and unit, by concept.
struct Slot<'a> {
    entity: EntityId,
    period: Period,
    unit: Unit,
    dimensions: Dimensions,
    values: BTreeMap<CanonicalElementKind, &'a StandardizedFact>,
    blocked: BTreeSet<CanonicalElementKind>,
}

impl<'a> Slot<'a> {
    /// Returns the slots of `series` and `added`, sorted by their key.
    ///
    /// A slot holds the values of one series key and one period type. So values whose periods
    /// differ only in the start share a slot, as in the derivation from a closed group. An instant
    /// and a duration of 0 quarters with the same end get separate slots. The slot takes the
    /// period of its first value.
    fn all(series: &'a Series, added: &'a [StandardizedFact]) -> Vec<Self> {
        let mut slots: BTreeMap<SlotKey<'a>, Self> = BTreeMap::new();
        for fact in series
            .facts
            .iter()
            .chain(added)
            .filter(|fact| fact.dimensions.is_empty())
        {
            let key = (
                &fact.entity,
                fact.period.end(),
                fact.period.quarters(),
                fact.period.period_type(),
                &fact.unit,
                &fact.dimensions,
            );
            slots
                .entry(key)
                .or_insert_with(|| Self {
                    entity: fact.entity.clone(),
                    period: fact.period,
                    unit: fact.unit.clone(),
                    dimensions: fact.dimensions.clone(),
                    values: BTreeMap::new(),
                    blocked: BTreeSet::new(),
                })
                .values
                .insert(fact.concept, fact);
        }
        // A series key has no period type, so the blocked concept marks the slot of each type. A
        // concept belongs to one statement, so the mark in the other slot changes no check.
        for key in series
            .blocked
            .iter()
            .filter(|key| key.dimensions.is_empty())
        {
            for period_type in [PeriodType::Instant, PeriodType::Duration] {
                let slot_key = (
                    &key.entity,
                    key.period_end,
                    key.quarters,
                    period_type,
                    &key.unit,
                    &key.dimensions,
                );
                if let Some(slot) = slots.get_mut(&slot_key) {
                    slot.blocked.insert(key.concept);
                }
            }
        }
        slots.into_values().collect()
    }

    /// Returns a [`Finding::MixedStarts`] if two facts that [`Statement`] counts in `group` have
    /// different exact starts.
    fn check_starts(&self, group: Group) -> Option<Finding> {
        let concepts = mixed_starts(group, |concept| self.values.get(&concept).copied())?;
        Some(Finding::MixedStarts {
            entity: self.entity.clone(),
            period: self.period,
            unit: self.unit.clone(),
            dimensions: self.dimensions.clone(),
            concepts,
            reason: group.name,
        })
    }

    /// Checks the closed group `group`, and returns a [`Finding::RollUpMismatch`] if it fails.
    fn check_closed_group(&self, group: Group) -> Option<Finding> {
        let rule = roll_up_rule(group.name);
        if self.values.values().any(|fact| fact.rule == rule) {
            return None;
        }
        if mixed_starts(group, |concept| self.values.get(&concept).copied()).is_some() {
            return None;
        }
        let mut inputs = Vec::new();
        for member in members(group) {
            if self.blocked.contains(&member.concept) {
                return None;
            }
            match self.values.get(&member.concept) {
                Some(fact) => inputs.push((member.sign, *fact)),
                None if member.zero_if_absent => {}
                None => return None,
            }
        }
        let difference: BigDecimal = inputs
            .iter()
            .map(|(sign, fact)| signed(*sign, &fact.value))
            .sum();
        let facts: Vec<&StandardizedFact> = inputs.into_iter().map(|(_, fact)| fact).collect();
        exceeds_tolerance(&difference, &facts, TOLERANCE_BASIS_POINTS)
            .then(|| self.mismatch(RollUpCheck::Group(group.name), &facts))
    }

    /// Checks the balance sheet identity, and returns a [`Finding::RollUpMismatch`] if it fails.
    fn check_balance_sheet(&self) -> Option<Finding> {
        let assets = *self.values.get(&BalanceSheetConcept::Assets.element())?;
        let liabilities_and_equity = *self
            .values
            .get(&BalanceSheetConcept::LiabilitiesAndEquity.element())?;
        let difference = &assets.value - &liabilities_and_equity.value;
        let facts = [assets, liabilities_and_equity];
        exceeds_tolerance(&difference, &facts, TOLERANCE_BASIS_POINTS)
            .then(|| self.mismatch(RollUpCheck::BalanceSheet, &facts))
    }

    /// Checks the other line of each group of [`NEGATIVE_OTHER_GROUPS`], and returns a
    /// [`Finding::NegativeOther`] for each other line below zero.
    fn check_negative_others(&self) -> Vec<Finding> {
        let statements: Vec<Statement> =
            [StatementKind::BalanceSheet, StatementKind::IncomeStatement]
                .into_iter()
                .filter_map(|kind| self.statement(kind))
                .collect();
        let mut findings = Vec::new();
        for group in groups().filter(|group| NEGATIVE_OTHER_GROUPS.contains(&group.name)) {
            let other = Line::Other { group: group.name };
            let Some(value) = statements
                .iter()
                .flat_map(Statement::lines)
                .find(|line| line.line() == other)
                .and_then(|line| line.value())
            else {
                continue;
            };
            let facts = self.parent_and_children(*group);
            if value.is_negative() && exceeds_tolerance(value, &facts, TOLERANCE_BASIS_POINTS) {
                findings.push(Finding::NegativeOther {
                    entity: self.entity.clone(),
                    period: self.period,
                    unit: self.unit.clone(),
                    dimensions: self.dimensions.clone(),
                    concepts: facts.iter().map(|fact| fact.concept).collect(),
                    reason: group.name,
                    confidence: weakest(&facts),
                });
            }
        }
        findings
    }

    /// Builds the [`Statement`] of `kind` from the values of the slot that `kind` places, or
    /// returns `None` if the slot has no such value.
    fn statement(&self, kind: StatementKind) -> Option<Statement> {
        let facts = self
            .values
            .values()
            .copied()
            .filter(|fact| statement_of(fact.concept) == Some(kind));
        Statement::build(kind, facts).ok()
    }

    /// Returns the facts of the parent and of the children of `group`, in the order of
    /// [`members`]. A concept with no value has no fact.
    fn parent_and_children(&self, group: Group) -> Vec<&'a StandardizedFact> {
        members(group)
            .iter()
            .filter_map(|member| self.values.get(&member.concept).copied())
            .collect()
    }

    /// Returns the [`Finding::RollUpMismatch`] of the check `reason` on `facts`.
    fn mismatch(&self, reason: RollUpCheck, facts: &[&StandardizedFact]) -> Finding {
        Finding::RollUpMismatch {
            entity: self.entity.clone(),
            period: self.period,
            unit: self.unit.clone(),
            dimensions: self.dimensions.clone(),
            concepts: facts.iter().map(|fact| fact.concept).collect(),
            reason,
            confidence: weakest(facts),
        }
    }
}

/// Returns the statement of `concept`, or `None` for a concept of a disclosure.
const fn statement_of(concept: CanonicalElementKind) -> Option<StatementKind> {
    match concept {
        CanonicalElementKind::Statement(StatementConcept::BalanceSheet(_)) => {
            Some(StatementKind::BalanceSheet)
        }
        CanonicalElementKind::Statement(StatementConcept::IncomeStatement(_)) => {
            Some(StatementKind::IncomeStatement)
        }
        CanonicalElementKind::Statement(StatementConcept::CashFlow(_)) => {
            Some(StatementKind::CashFlow)
        }
        CanonicalElementKind::Disclosure(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::candidate::SeriesKey;
    use crate::core::confidence::Confidence;
    use crate::core::elements::statement::cash_flow::CashFlowGroup;
    use crate::core::elements::{CashFlowConcept, IncomeStatementConcept};
    use crate::core::period::Duration;
    use crate::core::rule::RuleId;
    use crate::tests::fixtures::sample_fact::{date, in_cloud_segment, standardized};

    /// Returns the entity of [`standardized`].
    fn entity() -> EntityId {
        standardized(instant(), "0").entity
    }

    /// Returns the unit of [`standardized`].
    fn unit() -> Unit {
        standardized(instant(), "0").unit
    }

    fn instant() -> Period {
        Period::Instant {
            end: date(2026, 9, 30),
        }
    }

    fn quarter() -> Period {
        Period::Duration(Duration::from_end_and_quarters(date(2026, 9, 30), 1))
    }

    fn value(
        concept: impl Into<CanonicalElementKind>,
        period: Period,
        amount: &str,
    ) -> StandardizedFact {
        let mut fact = standardized(period, amount);
        fact.concept = concept.into();
        fact
    }

    /// Returns the duration from `start` to 2026-09-30.
    fn since(start: NaiveDate) -> Period {
        Period::Duration(
            Duration::from_dates(start, date(2026, 9, 30))
                .expect("Given a start before the end, the creation should always succeed"),
        )
    }

    /// Returns the 4-month stub that ends on 2026-09-30.
    fn four_months() -> Period {
        since(date(2026, 6, 1))
    }

    /// Returns the 3-month quarter that ends on 2026-09-30, with its exact start.
    fn three_months() -> Period {
        since(date(2026, 7, 1))
    }

    /// Returns the operating cash flow of the fixture filing: a 4-month total and share-based
    /// compensation, with net income and D&A of the quarter.
    fn operating_cash_flow() -> Vec<StandardizedFact> {
        vec![
            value(
                CashFlowConcept::OperatingCashFlow.element(),
                four_months(),
                "390",
            ),
            value(
                IncomeStatementConcept::NetIncome.element(),
                three_months(),
                "147",
            ),
            value(
                CashFlowConcept::DepreciationAmortization.element(),
                three_months(),
                "114",
            ),
            value(
                CashFlowConcept::ShareBasedCompensation.element(),
                four_months(),
                "16",
            ),
        ]
    }

    /// Returns the [`Finding::MixedStarts`] of `operating-cash-flow` for [`operating_cash_flow`].
    fn mixed_operating_cash_flow() -> Finding {
        Finding::MixedStarts {
            entity: entity(),
            period: four_months(),
            unit: unit(),
            dimensions: Dimensions::default(),
            concepts: vec![
                CashFlowConcept::OperatingCashFlow.element(),
                IncomeStatementConcept::NetIncome.element(),
                CashFlowConcept::DepreciationAmortization.element(),
                CashFlowConcept::ShareBasedCompensation.element(),
            ],
            reason: CashFlowGroup::OperatingCashFlow.name(),
        }
    }

    fn balance_sheet(concept: BalanceSheetConcept, amount: &str) -> StandardizedFact {
        value(concept.element(), instant(), amount)
    }

    fn cash_flow(concept: CashFlowConcept, amount: &str) -> StandardizedFact {
        value(concept.element(), quarter(), amount)
    }

    fn series(facts: Vec<StandardizedFact>) -> Series {
        Series {
            facts,
            blocked: Vec::new(),
        }
    }

    fn assets(non_current_assets: &str) -> Series {
        series(vec![
            balance_sheet(BalanceSheetConcept::Assets, "100000"),
            balance_sheet(BalanceSheetConcept::CurrentAssets, "60000"),
            balance_sheet(BalanceSheetConcept::NonCurrentAssets, non_current_assets),
        ])
    }

    fn mismatch(
        period: Period,
        concepts: Vec<CanonicalElementKind>,
        reason: RollUpCheck,
        confidence: Confidence,
    ) -> Finding {
        Finding::RollUpMismatch {
            entity: entity(),
            period,
            unit: unit(),
            dimensions: Dimensions::default(),
            concepts,
            reason,
            confidence,
        }
    }

    #[test]
    fn should_return_no_finding_when_a_closed_group_misses_by_exactly_the_tolerance() {
        let facts = assets("40100");

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_roll_up_mismatch_when_a_closed_group_misses_by_more_than_the_tolerance() {
        let facts = assets("40101");

        let expected_result = vec![mismatch(
            instant(),
            vec![
                BalanceSheetConcept::Assets.element(),
                BalanceSheetConcept::CurrentAssets.element(),
                BalanceSheetConcept::NonCurrentAssets.element(),
            ],
            RollUpCheck::Group(BalanceSheetGroup::Assets.name()),
            Confidence::Exact,
        )];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_roll_up_mismatch_when_the_values_carry_a_business_segment() {
        let facts = series(in_cloud_segment(assets("40101").facts));

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_the_check_of_a_closed_group_when_the_group_derived_a_value_for_the_period() {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::Assets, "100000"),
            balance_sheet(BalanceSheetConcept::CurrentAssets, "60000"),
        ]);
        let mut derived = balance_sheet(BalanceSheetConcept::NonCurrentAssets, "50000");
        derived.rule = RuleId::from_str("core/roll-up/assets@1").expect(
            "Given a hardcoded rule ID in the documented form, the parse should always succeed",
        );

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[derived]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_check_another_closed_group_when_one_group_derived_a_value_for_the_period() {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::Assets, "100000"),
            balance_sheet(BalanceSheetConcept::CurrentAssets, "60000"),
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "100000"),
            balance_sheet(BalanceSheetConcept::Liabilities, "50000"),
            balance_sheet(BalanceSheetConcept::Equity, "40000"),
        ]);
        let mut derived = balance_sheet(BalanceSheetConcept::NonCurrentAssets, "40000");
        derived.rule = RuleId::from_str("core/roll-up/assets@1").expect(
            "Given a hardcoded rule ID in the documented form, the parse should always succeed",
        );

        let expected_result = vec![mismatch(
            instant(),
            vec![
                BalanceSheetConcept::LiabilitiesAndEquity.element(),
                BalanceSheetConcept::Liabilities.element(),
                BalanceSheetConcept::Equity.element(),
            ],
            RollUpCheck::Group(BalanceSheetGroup::LiabilitiesAndEquity.name()),
            Confidence::Exact,
        )];

        let result = check_roll_ups(&facts, &[derived]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_check_values_together_when_their_periods_differ_only_in_the_start() {
        let facts = series(vec![
            value(
                IncomeStatementConcept::NetIncome.element(),
                quarter(),
                "100",
            ),
            value(
                IncomeStatementConcept::NetIncomeAttributableToParent.element(),
                quarter(),
                "90",
            ),
        ]);
        let with_start = Period::Duration(
            Duration::from_dates(date(2026, 7, 1), date(2026, 9, 30))
                .expect("Given a start before the end, the creation should always succeed"),
        );
        let noncontrolling = value(
            IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest.element(),
            with_start,
            "10",
        );

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[noncontrolling]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_mixed_starts_when_a_step_starts_from_a_quarter_and_holds_4_month_items() {
        let facts = series(operating_cash_flow());

        let expected_result = vec![mixed_operating_cash_flow()];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_check_of_another_group_when_one_group_of_the_slot_mixes_starts() {
        let mut facts = operating_cash_flow();
        facts.extend([
            value(
                CashFlowConcept::NetChangeInCash.element(),
                four_months(),
                "200",
            ),
            value(
                CashFlowConcept::InvestingCashFlow.element(),
                four_months(),
                "-138",
            ),
            value(
                CashFlowConcept::FinancingCashFlow.element(),
                four_months(),
                "-114",
            ),
        ]);
        let facts = series(facts);

        let expected_result = vec![
            mixed_operating_cash_flow(),
            mismatch(
                four_months(),
                vec![
                    CashFlowConcept::NetChangeInCash.element(),
                    CashFlowConcept::OperatingCashFlow.element(),
                    CashFlowConcept::InvestingCashFlow.element(),
                    CashFlowConcept::FinancingCashFlow.element(),
                ],
                RollUpCheck::Group(CashFlowGroup::NetChangeInCash.name()),
                Confidence::Exact,
            ),
        ];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_mixed_starts_and_no_roll_up_mismatch_when_a_closed_group_has_two_starts() {
        let facts = series(vec![
            value(
                CashFlowConcept::NetChangeInCash.element(),
                three_months(),
                "200",
            ),
            value(
                CashFlowConcept::OperatingCashFlow.element(),
                four_months(),
                "390",
            ),
            value(
                CashFlowConcept::InvestingCashFlow.element(),
                four_months(),
                "-138",
            ),
            value(
                CashFlowConcept::FinancingCashFlow.element(),
                four_months(),
                "-114",
            ),
        ]);

        let expected_result = vec![Finding::MixedStarts {
            entity: entity(),
            period: three_months(),
            unit: unit(),
            dimensions: Dimensions::default(),
            concepts: vec![
                CashFlowConcept::NetChangeInCash.element(),
                CashFlowConcept::OperatingCashFlow.element(),
                CashFlowConcept::InvestingCashFlow.element(),
                CashFlowConcept::FinancingCashFlow.element(),
            ],
            reason: CashFlowGroup::NetChangeInCash.name(),
        }];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_mixed_starts_when_the_other_member_has_no_exact_start() {
        let facts = series(vec![
            value(
                CashFlowConcept::OperatingCashFlow.element(),
                four_months(),
                "390",
            ),
            value(
                IncomeStatementConcept::NetIncome.element(),
                quarter(),
                "147",
            ),
            value(
                CashFlowConcept::ShareBasedCompensation.element(),
                four_months(),
                "16",
            ),
        ]);

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_negative_other_when_a_zero_quarter_duration_ends_on_a_balance_sheet_date() {
        let stub = Period::Duration(Duration::from_end_and_quarters(date(2026, 9, 30), 0));
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::Assets, "100000"),
            value(
                IncomeStatementConcept::OperatingExpenses.element(),
                stub,
                "100",
            ),
            value(
                IncomeStatementConcept::SellingGeneralAdmin.element(),
                stub,
                "80",
            ),
            value(
                IncomeStatementConcept::ResearchDevelopment.element(),
                stub,
                "30",
            ),
        ]);

        let expected_result = vec![Finding::NegativeOther {
            entity: entity(),
            period: stub,
            unit: unit(),
            dimensions: Dimensions::default(),
            concepts: vec![
                IncomeStatementConcept::OperatingExpenses.element(),
                IncomeStatementConcept::SellingGeneralAdmin.element(),
                IncomeStatementConcept::ResearchDevelopment.element(),
            ],
            reason: IncomeStatementGroup::OperatingExpenses.name(),
            confidence: Confidence::Exact,
        }];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_the_check_of_a_closed_group_when_dedup_blocks_a_member() {
        let mut facts = assets("50000");
        let mut blocked: SeriesKey = facts.facts[2].series_key();
        blocked.concept = BalanceSheetConcept::TemporaryEquity.element();
        facts.blocked = vec![blocked];
        facts.facts.truncate(2);
        facts.facts.extend([
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "100000"),
            balance_sheet(BalanceSheetConcept::Liabilities, "50000"),
            balance_sheet(BalanceSheetConcept::Equity, "40000"),
        ]);

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_the_weakest_confidence_when_a_closed_group_fails() {
        let mut facts = assets("45000");
        facts.facts[1].confidence = Confidence::Synonym;
        facts.facts[2].confidence = Confidence::Derived;

        let expected_result = vec![mismatch(
            instant(),
            vec![
                BalanceSheetConcept::Assets.element(),
                BalanceSheetConcept::CurrentAssets.element(),
                BalanceSheetConcept::NonCurrentAssets.element(),
            ],
            RollUpCheck::Group(BalanceSheetGroup::Assets.name()),
            Confidence::Derived,
        )];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_a_missing_exchange_rate_effect_as_zero_when_net_change_in_cash_is_checked() {
        let facts = series(vec![
            cash_flow(CashFlowConcept::NetChangeInCash, "137"),
            cash_flow(CashFlowConcept::OperatingCashFlow, "390"),
            cash_flow(CashFlowConcept::InvestingCashFlow, "-138"),
            cash_flow(CashFlowConcept::FinancingCashFlow, "-114"),
        ]);

        let expected_result = vec![mismatch(
            quarter(),
            vec![
                CashFlowConcept::NetChangeInCash.element(),
                CashFlowConcept::OperatingCashFlow.element(),
                CashFlowConcept::InvestingCashFlow.element(),
                CashFlowConcept::FinancingCashFlow.element(),
            ],
            RollUpCheck::Group(CashFlowGroup::NetChangeInCash.name()),
            Confidence::Exact,
        )];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_finding_when_the_balance_sheet_identity_misses_by_exactly_the_tolerance() {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::Assets, "1000000"),
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "1001001"),
        ]);

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_roll_up_mismatch_when_the_balance_sheet_identity_misses_by_more_than_the_tolerance()
     {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::Assets, "1000000"),
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "1001002"),
        ]);

        let expected_result = vec![mismatch(
            instant(),
            vec![
                BalanceSheetConcept::Assets.element(),
                BalanceSheetConcept::LiabilitiesAndEquity.element(),
            ],
            RollUpCheck::BalanceSheet,
            Confidence::Exact,
        )];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    fn current_assets(receivables: &str) -> Series {
        series(vec![
            balance_sheet(BalanceSheetConcept::CurrentAssets, "1000"),
            balance_sheet(BalanceSheetConcept::CashAndEquivalents, "600"),
            balance_sheet(BalanceSheetConcept::AccountsReceivable, receivables),
        ])
    }

    #[test]
    fn should_return_no_finding_when_the_other_line_falls_below_zero_by_exactly_the_tolerance() {
        let facts = current_assets("401");

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_negative_other_when_the_other_line_falls_below_zero_by_more_than_the_tolerance()
     {
        let facts = current_assets("401.5");

        let expected_result = vec![Finding::NegativeOther {
            entity: entity(),
            period: instant(),
            unit: unit(),
            dimensions: Dimensions::default(),
            concepts: vec![
                BalanceSheetConcept::CurrentAssets.element(),
                BalanceSheetConcept::CashAndEquivalents.element(),
                BalanceSheetConcept::AccountsReceivable.element(),
            ],
            reason: BalanceSheetGroup::CurrentAssets.name(),
            confidence: Confidence::Exact,
        }];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_no_negative_other_when_the_values_carry_a_business_segment() {
        let facts = series(in_cloud_segment(current_assets("401.5").facts));

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_report_a_negative_other_when_operating_expenses_fall_below_their_children() {
        let facts = series(vec![
            value(
                IncomeStatementConcept::OperatingExpenses.element(),
                quarter(),
                "100",
            ),
            value(
                IncomeStatementConcept::SellingGeneralAdmin.element(),
                quarter(),
                "80",
            ),
            value(
                IncomeStatementConcept::ResearchDevelopment.element(),
                quarter(),
                "30",
            ),
        ]);

        let expected_result = vec![Finding::NegativeOther {
            entity: entity(),
            period: quarter(),
            unit: unit(),
            dimensions: Dimensions::default(),
            concepts: vec![
                IncomeStatementConcept::OperatingExpenses.element(),
                IncomeStatementConcept::SellingGeneralAdmin.element(),
                IncomeStatementConcept::ResearchDevelopment.element(),
            ],
            reason: IncomeStatementGroup::OperatingExpenses.name(),
            confidence: Confidence::Exact,
        }];

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_finding_when_the_other_line_of_equity_falls_below_zero() {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "100"),
            balance_sheet(BalanceSheetConcept::RetainedEarnings, "150"),
        ]);

        let expected_result: Vec<Finding> = Vec::new();

        let result = check_roll_ups(&facts, &[]);

        assert_eq!(result, expected_result);
    }
}
