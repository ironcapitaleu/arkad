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
/// the cash flow statement. Each statement keeps the order of its table in `xbrl/DESIGN.md`,
/// "Roll-Up Tree". The derivation from a closed group visits the groups in this order.
///
/// The list holds a group only if each member is a standard concept. So a placement can name a
/// group that the list does not hold, such as `liabilities-and-equity`.
pub fn groups() -> impl Iterator<Item = &'static Group> {
    balance_sheet::GROUPS
        .iter()
        .chain(income_statement::GROUPS)
        .chain(cash_flow::GROUPS)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::{Concept, Placed, Placement};

    #[test]
    fn should_join_the_groups_in_the_order_of_the_statements_and_their_tables() {
        let expected_result = vec![
            "assets",
            "current-assets",
            "non-current-assets",
            "liabilities",
            "non-current-liabilities",
            "gross-profit",
            "comprehensive-income",
            "operating-cash-flow",
            "investing-cash-flow",
            "financing-cash-flow",
        ];

        let result: Vec<&str> = groups().map(|group| group.name).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_every_group_at_least_one_child_when_reading_the_placements() {
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

        let expected_result: Vec<&str> = Vec::new();

        let result: Vec<&str> = groups()
            .map(|group| group.name)
            .filter(|name| {
                !placements.iter().any(|placement| {
                    matches!(placement, Placement::Child { group, .. } if group == name)
                })
            })
            .collect();

        assert_eq!(result, expected_result);
    }
}
