//! # Statement Concepts
//!
//! Provides the concept enums of the three statements and [`groups`], the groups of the roll-up
//! tree.
//!
//! Each statement module holds the placements of its concepts and the groups whose parent it
//! holds.
//!
//! ## Modules
//!
//! - [`balance_sheet`]: The concepts of the balance sheet and their groups.
//! - [`income_statement`]: The concepts of the income statement and their groups.
//! - [`cash_flow`]: The concepts of the cash flow statement and their groups.

use super::roll_up::Group;

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
/// groups in this order. `docs/adr/0013-order-of-the-groups.md` records the order.
///
/// The list holds a group only if its parent, its start if it is a step, and at least one child
/// are standard concepts.
pub fn groups() -> impl Iterator<Item = &'static Group> {
    balance_sheet::GROUPS
        .iter()
        .chain(income_statement::GROUPS)
        .chain(cash_flow::GROUPS)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::{Concept, GroupName, Placed, Placement};

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
        let placements: Vec<Placement> = BalanceSheetConcept::ALL
            .iter()
            .map(|concept| concept.placement())
            .chain(
                IncomeStatementConcept::ALL
                    .iter()
                    .map(|concept| concept.placement()),
            )
            .chain(
                CashFlowConcept::ALL
                    .iter()
                    .map(|concept| concept.placement()),
            )
            .collect();

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
