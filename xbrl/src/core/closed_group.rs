//! # Closed Group
//!
//! Provides [`derive_from_closed_groups`], the rules `core/roll-up/<group>@1`: a closed group of the
//! roll-up tree with exactly one unknown member gives that member.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use bigdecimal::BigDecimal;
use jiff::civil::Date;

use super::candidate::SeriesKey;
use super::confidence::Confidence;
use super::dedup::Series;
use super::elements::statement::{groups, placements};
use super::elements::{
    CanonicalElementKind, CashFlowConcept, Group, GroupKind, GroupName, IncomeStatementConcept,
    Placement, Weight,
};
use super::fact::Dimensions;
use super::filing::EntityId;
use super::rule::RuleId;
use super::standardized::StandardizedFact;
use super::statement::mixed_starts;

/// The concepts that no group derives.
///
/// `gross-profit` never derives `GrossProfit`. A single-step filer chooses not to present gross
/// profit, and its cost of revenue can leave out depreciation.
const NEVER_DERIVED: [CanonicalElementKind; 1] = [IncomeStatementConcept::GrossProfit.element()];

/// The children that are zero if absent only in the check, not in the derivation.
///
/// Filings before 2018 report the exchange rate effect under
/// `EffectOfExchangeRateOnCashAndCashEquivalents`, which leaves out restricted cash and is no
/// alias. If the effect counted as zero, the rule would derive a wrong `NetChangeInCash` for every
/// multinational before 2018.
const ZERO_IN_CHECK_ONLY: [CanonicalElementKind; 1] =
    [CashFlowConcept::ExchangeRateEffect.element()];

/// Applies the rules `core/roll-up/<group>@1` to the series of one entity, and returns the derived
/// facts.
///
/// `fourth_quarters` holds the facts of `core/period/q4-from-fy@1` for the same series. The rule
/// reads them as it reads `series.facts`.
///
/// A closed group with exactly one unknown member gives that member: the parent, the start of a
/// step, or a child. The members satisfy parent = start + Σ(weight × child). A member is unknown if
/// the series holds no value for it at the period. A child that is zero if absent counts as known
/// with zero, except `ExchangeRateEffect`. Open groups derive nothing, and `gross-profit` never
/// derives `GrossProfit`.
///
/// The rule reads only values without dimensions. The identities of the tree do not hold for the
/// value of one segment.
///
/// The group derives nothing for a period if a member is blocked by a dedup conflict, has values in
/// two units, or has another unit than the other members. It also derives nothing if two facts
/// that [`Statement`](super::statement::Statement) counts in the group have different exact
/// starts, such as a quarter and a 4-month stub. A member with no value counts as the members of
/// the group that makes it, so the group derives nothing even if its own inputs share one start.
/// [`check_roll_ups`](super::roll_up_check::check_roll_ups) reports such a group.
///
/// The rule repeats until no group gives a new value, so a derived value can feed another group. In
/// each pass, it visits the closed groups in the order of
/// [`groups`]. The first group that gives a value wins.
/// `docs/adr/0013-order-of-the-groups.md` records the order.
///
/// The fact takes the period, the unit, and the release of its first known member. The confidence
/// is [`Confidence::Derived`], or the weakest confidence of an input. The fact has `restated` if an
/// input has it.
#[must_use]
pub fn derive_from_closed_groups(
    series: &Series,
    fourth_quarters: &[StandardizedFact],
) -> Vec<StandardizedFact> {
    let closed: Vec<(&Group, Vec<Member>)> = groups()
        .filter(|group| group.closed)
        .map(|group| (group, members(*group)))
        .collect();
    let mut known = KnownValues::of(series, fourth_quarters);
    let mut derived = Vec::new();
    loop {
        let before = derived.len();
        for (group, members) in &closed {
            for slot in known.slots() {
                if let Some(fact) = known.derive(group, members, &slot) {
                    known.insert(slot, fact.clone());
                    derived.push(fact);
                }
            }
        }
        if derived.len() == before {
            return derived;
        }
    }
}

/// Returns the ID of the rule that derives a member of `group`, such as
/// `core/roll-up/liabilities@1`.
pub(super) fn roll_up_rule(group: GroupName) -> RuleId {
    RuleId::from_str(&format!("core/roll-up/{group}@1"))
        .expect("Given a kebab-case group name without whitespace, the rule ID should always parse")
}

/// One member of a group, with its sign in the sum that equals zero: parent − start − Σ(weight ×
/// child) = 0.
///
/// `zero_if_absent` is the field of the placement. The derivation leaves out the children of
/// [`ZERO_IN_CHECK_ONLY`].
#[derive(Debug)]
pub(super) struct Member {
    pub(super) concept: CanonicalElementKind,
    pub(super) sign: Weight,
    pub(super) zero_if_absent: bool,
}

/// Returns the parent, the start of a step, and the children of `group`, in this order. The
/// children keep the order of [`placements`].
pub(super) fn members(group: Group) -> Vec<Member> {
    let parent = Member {
        concept: group.parent,
        sign: Weight::Plus,
        zero_if_absent: false,
    };
    let start = match group.kind {
        GroupKind::Step { start } => Some(Member {
            concept: start,
            sign: Weight::Minus,
            zero_if_absent: false,
        }),
        GroupKind::Breakdown => None,
    };
    let children = placements().filter_map(|(concept, placement)| match placement {
        Placement::Child {
            group: name,
            weight,
            zero_if_absent,
        } if name == group.name => Some(Member {
            concept,
            sign: opposite(weight),
            zero_if_absent,
        }),
        _ => None,
    });
    std::iter::once(parent)
        .chain(start)
        .chain(children)
        .collect()
}

const fn opposite(weight: Weight) -> Weight {
    match weight {
        Weight::Minus => Weight::Plus,
        Weight::Plus => Weight::Minus,
    }
}

/// Returns `value` with the sign `sign`.
pub(super) fn signed(sign: Weight, value: &BigDecimal) -> BigDecimal {
    match sign {
        Weight::Minus => -value,
        Weight::Plus => value.clone(),
    }
}

/// The period of a value without dimensions. The slot leaves out the concept and the unit. Each
/// member of a group has one value per slot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Slot {
    entity: EntityId,
    period_end: Date,
    quarters: u16,
    dimensions: Dimensions,
}

impl Slot {
    fn of(key: SeriesKey) -> Self {
        Self {
            entity: key.entity,
            period_end: key.period_end,
            quarters: key.quarters,
            dimensions: key.dimensions,
        }
    }
}

/// The values of each slot by concept, and the concepts that dedup blocks in each slot.
#[derive(Debug)]
struct KnownValues {
    values: BTreeMap<Slot, BTreeMap<CanonicalElementKind, Vec<StandardizedFact>>>,
    blocked: BTreeMap<Slot, BTreeSet<CanonicalElementKind>>,
}

impl KnownValues {
    fn of(series: &Series, fourth_quarters: &[StandardizedFact]) -> Self {
        let mut known = Self {
            values: BTreeMap::new(),
            blocked: BTreeMap::new(),
        };
        for key in series
            .blocked
            .iter()
            .filter(|key| key.dimensions.is_empty())
        {
            let concept = key.concept;
            known
                .blocked
                .entry(Slot::of(key.clone()))
                .or_default()
                .insert(concept);
        }
        for fact in series
            .facts
            .iter()
            .chain(fourth_quarters)
            .filter(|fact| fact.dimensions.is_empty())
        {
            known.insert(Slot::of(fact.series_key()), fact.clone());
        }
        known
    }

    fn slots(&self) -> Vec<Slot> {
        self.values.keys().cloned().collect()
    }

    fn insert(&mut self, slot: Slot, fact: StandardizedFact) {
        self.values
            .entry(slot)
            .or_default()
            .entry(fact.concept)
            .or_default()
            .push(fact);
    }

    /// Returns the value that `group` with `members` gives for its one unknown member in `slot`,
    /// or `None` if the group gives no value.
    fn derive(&self, group: &Group, members: &[Member], slot: &Slot) -> Option<StandardizedFact> {
        let values = self.values.get(slot)?;
        let value_of = |concept| match values.get(&concept).map(Vec::as_slice) {
            Some([fact]) => Some(fact),
            _ => None,
        };
        if mixed_starts(*group, value_of).is_some() {
            return None;
        }
        let blocked = self.blocked.get(slot);
        let mut unknown = None;
        let mut inputs: Vec<(Weight, &StandardizedFact)> = Vec::new();
        for member in members {
            if blocked.is_some_and(|concepts| concepts.contains(&member.concept)) {
                return None;
            }
            match values.get(&member.concept).map(Vec::as_slice) {
                Some([fact]) => inputs.push((member.sign, fact)),
                None if member.zero_if_absent && !ZERO_IN_CHECK_ONLY.contains(&member.concept) => {}
                None if unknown.is_none() => unknown = Some(member),
                _ => return None,
            }
        }
        let unknown = unknown.filter(|member| !NEVER_DERIVED.contains(&member.concept))?;
        let (_, first) = *inputs.first()?;
        if inputs.iter().any(|(_, fact)| fact.unit != first.unit) {
            return None;
        }
        let sum: BigDecimal = inputs
            .iter()
            .map(|(sign, fact)| signed(*sign, &fact.value))
            .sum();
        Some(StandardizedFact {
            entity: first.entity.clone(),
            concept: unknown.concept,
            period: first.period,
            unit: first.unit.clone(),
            dimensions: first.dimensions.clone(),
            value: signed(opposite(unknown.sign), &sum),
            confidence: inputs
                .iter()
                .map(|(_, fact)| fact.confidence)
                .fold(Confidence::Derived, Confidence::max),
            rule: roll_up_rule(group.name),
            release: first.release.clone(),
            inputs: inputs
                .iter()
                .flat_map(|(_, fact)| fact.inputs.iter().cloned())
                .collect(),
            restated: inputs.iter().any(|(_, fact)| fact.restated),
            // Deviation: ADR 0049, decision 6, gives a derived value the scope of its members.
            // No alias or part names a scope, so no member carries one, and the value carries none.
            scope: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::BalanceSheetConcept;
    use crate::core::fact::FactKey;
    use crate::core::period::{Duration, Period};
    use crate::core::release::{Sha256, TaxonomyRelease};
    use crate::core::unit::Unit;
    use crate::tests::fixtures::sample_fact::{date, in_cloud_segment, standardized};

    /// The concept, value, confidence, and rule of a derived fact.
    type Summary = (CanonicalElementKind, BigDecimal, Confidence, String);

    fn instant() -> Period {
        Period::Instant {
            end: date(2026, 9, 30),
        }
    }

    fn quarter() -> Period {
        Period::Duration(
            Duration::from_dates(date(2026, 7, 1), date(2026, 9, 30))
                .expect("Given an end after the start, the creation should always succeed"),
        )
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
    fn since(start: Date) -> Period {
        Period::Duration(
            Duration::from_dates(start, date(2026, 9, 30))
                .expect("Given a start before the end, the creation should always succeed"),
        )
    }

    fn balance_sheet(concept: BalanceSheetConcept, amount: &str) -> StandardizedFact {
        value(concept.element(), instant(), amount)
    }

    fn series(facts: Vec<StandardizedFact>) -> Series {
        Series {
            facts,
            blocked: Vec::new(),
        }
    }

    fn summary(facts: Vec<StandardizedFact>) -> Vec<Summary> {
        facts
            .into_iter()
            .map(|fact| {
                (
                    fact.concept,
                    fact.value,
                    fact.confidence,
                    fact.rule.to_string(),
                )
            })
            .collect()
    }

    fn derived(concept: CanonicalElementKind, amount: i64, rule: &str) -> Summary {
        (
            concept,
            BigDecimal::from(amount),
            Confidence::Derived,
            rule.to_string(),
        )
    }

    #[test]
    fn should_derive_liabilities_when_temporary_equity_is_absent() {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "7216"),
            balance_sheet(BalanceSheetConcept::Equity, "-252"),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "-252"),
        ]);

        let expected_result = vec![derived(
            BalanceSheetConcept::Liabilities.element(),
            7468,
            "core/roll-up/liabilities-and-equity@1",
        )];

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_members_of_a_closed_group_carry_a_business_segment() {
        let facts = series(in_cloud_segment(vec![
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "7216"),
            balance_sheet(BalanceSheetConcept::Equity, "-252"),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "-252"),
        ]));

        let expected_result: Vec<Summary> = Vec::new();

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_non_current_liabilities_from_derived_liabilities_when_the_chain_holds_in_one_pass()
     {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "100"),
            balance_sheet(BalanceSheetConcept::Equity, "40"),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "40"),
            balance_sheet(BalanceSheetConcept::CurrentLiabilities, "25"),
        ]);

        let expected_result = vec![
            derived(
                BalanceSheetConcept::Liabilities.element(),
                60,
                "core/roll-up/liabilities-and-equity@1",
            ),
            derived(
                BalanceSheetConcept::NonCurrentLiabilities.element(),
                35,
                "core/roll-up/liabilities@1",
            ),
        ];

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_liabilities_from_derived_equity_when_a_later_group_feeds_an_earlier_group() {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "100"),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "40"),
        ]);

        let expected_result = vec![
            derived(
                BalanceSheetConcept::Equity.element(),
                40,
                "core/roll-up/equity@1",
            ),
            derived(
                BalanceSheetConcept::Liabilities.element(),
                60,
                "core/roll-up/liabilities-and-equity@1",
            ),
        ];

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_liabilities_from_the_first_group_when_two_closed_groups_can_derive_it() {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "100"),
            balance_sheet(BalanceSheetConcept::Equity, "40"),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "40"),
            balance_sheet(BalanceSheetConcept::CurrentLiabilities, "10"),
            balance_sheet(BalanceSheetConcept::NonCurrentLiabilities, "20"),
        ]);

        let expected_result = vec![derived(
            BalanceSheetConcept::Liabilities.element(),
            60,
            "core/roll-up/liabilities-and-equity@1",
        )];

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_net_income_when_the_noncontrolling_interest_is_absent() {
        let facts = series(vec![value(
            IncomeStatementConcept::NetIncomeAttributableToParent.element(),
            quarter(),
            "230",
        )]);

        let expected_result = vec![derived(
            IncomeStatementConcept::NetIncome.element(),
            230,
            "core/roll-up/net-income-attribution@1",
        )];

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_exchange_rate_effect_is_absent() {
        let facts = series(vec![
            value(
                CashFlowConcept::OperatingCashFlow.element(),
                quarter(),
                "390",
            ),
            value(
                CashFlowConcept::InvestingCashFlow.element(),
                quarter(),
                "-138",
            ),
            value(
                CashFlowConcept::FinancingCashFlow.element(),
                quarter(),
                "-114",
            ),
        ]);

        let expected_result: Vec<Summary> = Vec::new();

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_net_change_in_cash_when_the_exchange_rate_effect_is_known() {
        let facts = series(vec![
            value(
                CashFlowConcept::OperatingCashFlow.element(),
                quarter(),
                "390",
            ),
            value(
                CashFlowConcept::InvestingCashFlow.element(),
                quarter(),
                "-138",
            ),
            value(
                CashFlowConcept::FinancingCashFlow.element(),
                quarter(),
                "-114",
            ),
            value(
                CashFlowConcept::ExchangeRateEffect.element(),
                quarter(),
                "-1",
            ),
        ]);

        let expected_result = vec![derived(
            CashFlowConcept::NetChangeInCash.element(),
            137,
            "core/roll-up/net-change-in-cash@1",
        )];

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_members_of_a_closed_group_have_two_exact_starts() {
        let four_months = since(date(2026, 6, 1));
        let facts = series(vec![
            value(
                CashFlowConcept::OperatingCashFlow.element(),
                since(date(2026, 7, 1)),
                "390",
            ),
            value(
                CashFlowConcept::InvestingCashFlow.element(),
                four_months,
                "-138",
            ),
            value(
                CashFlowConcept::FinancingCashFlow.element(),
                four_months,
                "-114",
            ),
            value(
                CashFlowConcept::ExchangeRateEffect.element(),
                four_months,
                "-1",
            ),
        ]);

        let expected_result: Vec<Summary> = Vec::new();

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_no_operating_cash_flow_when_the_children_of_the_unknown_member_have_two_exact_starts()
     {
        let four_months = since(date(2026, 6, 1));
        let facts = series(vec![
            value(
                CashFlowConcept::NetChangeInCash.element(),
                four_months,
                "137",
            ),
            value(
                CashFlowConcept::InvestingCashFlow.element(),
                four_months,
                "-138",
            ),
            value(
                CashFlowConcept::FinancingCashFlow.element(),
                four_months,
                "-114",
            ),
            value(
                CashFlowConcept::ExchangeRateEffect.element(),
                four_months,
                "-1",
            ),
            value(
                IncomeStatementConcept::NetIncome.element(),
                since(date(2026, 7, 1)),
                "147",
            ),
            value(
                CashFlowConcept::ShareBasedCompensation.element(),
                four_months,
                "16",
            ),
        ]);

        // Only `net-income-attribution` derives. `net-change-in-cash` counts the children of the
        // missing operating cash flow, which have two starts.
        let expected_result = vec![derived(
            IncomeStatementConcept::NetIncomeAttributableToParent.element(),
            147,
            "core/roll-up/net-income-attribution@1",
        )];

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_gross_profit_is_the_unknown_member() {
        let facts = series(vec![
            value(IncomeStatementConcept::Revenue.element(), quarter(), "1000"),
            value(
                IncomeStatementConcept::CostOfRevenue.element(),
                quarter(),
                "600",
            ),
        ]);

        let expected_result: Vec<Summary> = Vec::new();

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_the_cost_of_revenue_when_revenue_and_gross_profit_are_known() {
        let facts = series(vec![
            value(IncomeStatementConcept::Revenue.element(), quarter(), "1000"),
            value(
                IncomeStatementConcept::GrossProfit.element(),
                quarter(),
                "400",
            ),
        ]);

        let expected_result = vec![derived(
            IncomeStatementConcept::CostOfRevenue.element(),
            600,
            "core/roll-up/gross-profit@1",
        )];

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_revenue_when_gross_profit_and_the_cost_of_revenue_are_known() {
        let facts = series(vec![
            value(
                IncomeStatementConcept::GrossProfit.element(),
                quarter(),
                "400",
            ),
            value(
                IncomeStatementConcept::CostOfRevenue.element(),
                quarter(),
                "600",
            ),
        ]);

        let expected_result = vec![derived(
            IncomeStatementConcept::Revenue.element(),
            1000,
            "core/roll-up/gross-profit@1",
        )];

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_the_group_is_open() {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::CashAndEquivalents, "388"),
            balance_sheet(BalanceSheetConcept::ShortTermInvestments, "0"),
            balance_sheet(BalanceSheetConcept::AccountsReceivable, "1262"),
            balance_sheet(BalanceSheetConcept::Inventory, "0"),
        ]);

        let expected_result: Vec<Summary> = Vec::new();

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_two_members_have_different_units() {
        let mut equity = balance_sheet(BalanceSheetConcept::Equity, "-252");
        equity.unit = Unit::from_str("EUR")
            .expect("Given a hardcoded unit with one measure, the parse should always succeed");
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "7216"),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "-252"),
            equity,
        ]);

        let expected_result: Vec<Summary> = Vec::new();

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_a_member_has_values_in_two_units() {
        let mut equity_in_euro = balance_sheet(BalanceSheetConcept::Equity, "-230");
        equity_in_euro.unit = Unit::from_str("EUR")
            .expect("Given a hardcoded unit with one measure, the parse should always succeed");
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "7216"),
            balance_sheet(BalanceSheetConcept::Equity, "-252"),
            equity_in_euro,
        ]);

        let expected_result: Vec<Summary> = Vec::new();

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_mark_the_derived_fact_restated_when_an_input_is_restated() {
        let mut equity = balance_sheet(BalanceSheetConcept::Equity, "-252");
        equity.restated = true;
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "7216"),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "-252"),
            equity,
        ]);

        let expected_result = vec![true];

        let result: Vec<bool> = derive_from_closed_groups(&facts, &[])
            .into_iter()
            .map(|fact| fact.restated)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_inputs_of_every_known_member_when_it_derives_a_member() {
        let liabilities_and_equity =
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "7216");
        let mut equity = balance_sheet(BalanceSheetConcept::Equity, "-252");
        equity.inputs[0].copy_rank = 1;
        let facts = series(vec![
            liabilities_and_equity.clone(),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "-252"),
            equity.clone(),
        ]);

        let expected_result = vec![vec![
            liabilities_and_equity.inputs[0].clone(),
            equity.inputs[0].clone(),
        ]];

        let result: Vec<Vec<FactKey>> = derive_from_closed_groups(&facts, &[])
            .into_iter()
            .map(|fact| fact.inputs)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_release_of_the_first_known_member_when_it_derives_a_member() {
        let liabilities_and_equity =
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "7216");
        let mut equity = balance_sheet(BalanceSheetConcept::Equity, "-252");
        equity.release.checksum = Sha256::new([1; 32]);
        let facts = series(vec![
            liabilities_and_equity.clone(),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "-252"),
            equity,
        ]);

        let expected_result = vec![liabilities_and_equity.release];

        let result: Vec<TaxonomyRelease> = derive_from_closed_groups(&facts, &[])
            .into_iter()
            .map(|fact| fact.release)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_dedup_blocks_the_zero_if_absent_child() {
        let liabilities_and_equity =
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "7216");
        let mut temporary_equity = liabilities_and_equity.series_key();
        temporary_equity.concept = BalanceSheetConcept::TemporaryEquity.element();
        let facts = Series {
            facts: vec![
                liabilities_and_equity,
                balance_sheet(BalanceSheetConcept::Equity, "-252"),
                balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "-252"),
            ],
            blocked: vec![temporary_equity],
        };

        let expected_result: Vec<Summary> = Vec::new();

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_nothing_when_every_member_is_known() {
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::Assets, "7216"),
            balance_sheet(BalanceSheetConcept::CurrentAssets, "1846"),
            balance_sheet(BalanceSheetConcept::NonCurrentAssets, "5000"),
        ]);

        let expected_result: Vec<Summary> = Vec::new();

        let result = summary(derive_from_closed_groups(&facts, &[]));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_take_the_weakest_confidence_when_an_input_is_computed() {
        let mut equity = balance_sheet(BalanceSheetConcept::Equity, "-252");
        equity.confidence = Confidence::Computed;
        let facts = series(vec![
            balance_sheet(BalanceSheetConcept::LiabilitiesAndEquity, "7216"),
            balance_sheet(BalanceSheetConcept::EquityAttributableToParent, "-252"),
            equity,
        ]);

        let expected_result = vec![Confidence::Computed];

        let result: Vec<Confidence> = derive_from_closed_groups(&facts, &[])
            .into_iter()
            .map(|fact| fact.confidence)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_derive_from_a_fourth_quarter_when_the_period_identity_gave_it() {
        let facts = series(Vec::new());
        let fourth_quarters = vec![value(
            IncomeStatementConcept::NetIncomeAttributableToParent.element(),
            quarter(),
            "230",
        )];

        let expected_result = vec![derived(
            IncomeStatementConcept::NetIncome.element(),
            230,
            "core/roll-up/net-income-attribution@1",
        )];

        let result = summary(derive_from_closed_groups(&facts, &fourth_quarters));

        assert_eq!(result, expected_result);
    }
}
