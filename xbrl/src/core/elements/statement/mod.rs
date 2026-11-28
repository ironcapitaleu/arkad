//! # Statement Concepts
//!
//! Provides the concept enums of the three statements, [`groups`], the groups of the roll-up tree,
//! and [`placements`], the place of each concept in the tree.
//!
//! Each statement module holds the placements of its concepts and the groups whose parent it
//! holds.
//!
//! ## Modules
//!
//! - [`balance_sheet`]: The concepts of the balance sheet and their groups.
//! - [`income_statement`]: The concepts of the income statement and their groups.
//! - [`cash_flow`]: The concepts of the cash flow statement and their groups.

use super::roll_up::{Group, Placement};
use super::{CanonicalElementKind, Concept, Placed};

pub mod balance_sheet;
pub mod cash_flow;
pub mod income_statement;

pub use balance_sheet::BalanceSheetConcept;
pub use cash_flow::CashFlowConcept;
pub use income_statement::IncomeStatementConcept;

/// Returns every group of the roll-up tree.
///
/// The groups of the balance sheet come first, then those of the income statement, then those of
/// the cash flow statement. Each statement lists its groups in the order of the first row that
/// names each group in the table of the statement. The derivation from a closed group visits the
/// groups in this order.
///
/// The list holds a group only if its parent, its start if it is a step, and at least one child
/// are standard concepts.
pub fn groups() -> impl Iterator<Item = &'static Group> {
    balance_sheet::GROUPS
        .iter()
        .chain(income_statement::GROUPS)
        .chain(cash_flow::GROUPS)
}

/// Returns each concept of the three statements with its placement.
///
/// The concepts of the balance sheet come first, then those of the income statement, then those of
/// the cash flow statement. Each statement keeps the order of the variants of its enum.
pub fn placements() -> impl Iterator<Item = (CanonicalElementKind, Placement)> {
    let balance_sheet = BalanceSheetConcept::ALL
        .iter()
        .map(|&concept| (concept.element(), concept.placement()));
    let income_statement = IncomeStatementConcept::ALL
        .iter()
        .map(|&concept| (concept.element(), concept.placement()));
    let cash_flow = CashFlowConcept::ALL
        .iter()
        .map(|&concept| (concept.element(), concept.placement()));
    balance_sheet.chain(income_statement).chain(cash_flow)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::GroupName;

    #[test]
    fn should_join_the_groups_in_the_order_of_the_statements_and_their_tables() {
        let expected_result = vec![
            "assets",
            "current-assets",
            "non-current-assets",
            "liabilities-and-equity",
            "liabilities",
            "current-liabilities",
            "non-current-liabilities",
            "equity",
            "equity-attributable-to-parent",
            "gross-profit",
            "operating-income",
            "operating-expenses",
            "pretax-income",
            "net-income",
            "net-income-attribution",
            "comprehensive-income",
            "net-change-in-cash",
            "operating-cash-flow",
            "investing-cash-flow",
            "financing-cash-flow",
        ];

        let result: Vec<&str> = groups().map(|group| group.name.as_str()).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_at_least_one_child_for_every_listed_group() {
        let placements: Vec<Placement> = placements().map(|(_, placement)| placement).collect();

        let expected_result: Vec<GroupName> = Vec::new();

        let result: Vec<GroupName> = groups()
            .map(|group| group.name)
            .filter(|name| {
                !placements.iter().any(|placement| {
                    matches!(placement, Placement::Child { group, .. } if group == name)
                })
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_four_children_as_zero_if_absent_for_the_concepts_of_the_statements() {
        let concepts = placements();

        let expected_result = vec![
            BalanceSheetConcept::TemporaryEquity.element(),
            BalanceSheetConcept::NoncontrollingInterest.element(),
            IncomeStatementConcept::NetIncomeAttributableToNoncontrollingInterest.element(),
            CashFlowConcept::ExchangeRateEffect.element(),
        ];

        let result: Vec<CanonicalElementKind> = concepts
            .filter(|(_, placement)| placement.is_zero_if_absent())
            .map(|(concept, _)| concept)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_place_every_zero_if_absent_child_in_a_closed_group_for_the_concepts_of_the_statements()
     {
        let concepts = placements();

        let expected_result: Vec<GroupName> = Vec::new();

        let result: Vec<GroupName> = concepts
            .filter_map(|(_, placement)| match placement {
                Placement::Child {
                    group,
                    zero_if_absent: true,
                    ..
                } => Some(group),
                _ => None,
            })
            .filter(|name| !groups().any(|group| group.name == *name && group.closed))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_each_group_a_unique_name_for_every_listed_group() {
        let names: Vec<&str> = groups().map(|group| group.name.as_str()).collect();

        let expected_result = (20, 20);

        let result = (names.len(), names.iter().collect::<BTreeSet<_>>().len());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_distinct_name_to_each_of_the_20_group_names() {
        let names: Vec<&str> = balance_sheet::BalanceSheetGroup::ALL
            .iter()
            .map(|group| group.as_str())
            .chain(
                income_statement::IncomeStatementGroup::ALL
                    .iter()
                    .map(|group| group.as_str()),
            )
            .chain(
                cash_flow::CashFlowGroup::ALL
                    .iter()
                    .map(|group| group.as_str()),
            )
            .collect();

        let expected_result = (20, 20);

        let result = (names.len(), names.iter().collect::<BTreeSet<_>>().len());

        assert_eq!(result, expected_result);
    }
}
