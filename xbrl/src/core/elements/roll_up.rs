//! # Roll-Up Tree
//!
//! Provides the types of the roll-up tree: the [`Placement`] of a concept, its [`Weight`], the
//! [`Group`] that sums children into a parent, and the [`GroupName`] of a group.
//!
//! Each statement module holds the placements of its concepts and the groups whose parent it
//! holds.

use std::fmt;

use super::CanonicalElementKind;
use super::statement::balance_sheet::BalanceSheetGroup;
use super::statement::cash_flow::CashFlowGroup;
use super::statement::income_statement::IncomeStatementGroup;

/// The place of a concept in the roll-up tree of its statement.
///
/// The statement comes from the place of the concept in [`CanonicalElementKind`], so the
/// placement names none.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Placement {
    /// A line on the top level of its statement, such as `Assets` or `Revenue`.
    Top,
    /// A child of one group.
    #[non_exhaustive]
    Child {
        /// The group the child belongs to.
        group: GroupName,
        /// The sign with which the child adds to the parent.
        weight: Weight,
        /// `true` if a closed group uses zero for the child when it has no value for a period.
        /// Most filers have no temporary equity, for example, and report no zero for it.
        zero_if_absent: bool,
    },
}

impl Placement {
    /// Creates a [`Placement::Child`] of the group `group` with the weight `weight`. The child
    /// counts as unknown when the filer reports no value.
    #[must_use]
    pub const fn child(group: GroupName, weight: Weight) -> Self {
        Self::Child {
            group,
            weight,
            zero_if_absent: false,
        }
    }

    /// Creates a [`Placement::Child`] of the group `group` with the weight `weight`. The child
    /// counts as zero when the filer reports no value.
    #[must_use]
    pub const fn child_zero_if_absent(group: GroupName, weight: Weight) -> Self {
        Self::Child {
            group,
            weight,
            zero_if_absent: true,
        }
    }

    /// Returns `true` if the placement is a child that counts as zero when it has no value.
    #[must_use]
    pub const fn is_zero_if_absent(self) -> bool {
        matches!(
            self,
            Self::Child {
                zero_if_absent: true,
                ..
            }
        )
    }
}

/// The sign with which a child adds to the parent of its group.
///
/// A value keeps the sign of the filing, and an expense or a payment is positive. So a cost that a
/// parent subtracts, such as `CostOfRevenue` in `gross-profit`, carries [`Weight::Minus`]. A cost
/// in a breakdown of costs, such as `SellingGeneralAdmin` in `operating-expenses`, carries
/// [`Weight::Plus`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weight {
    /// The child adds to the parent: `+1`.
    Plus,
    /// The child subtracts from the parent: `−1`.
    Minus,
}

/// One sum of the roll-up tree: a parent and the concepts that add up to it.
///
/// The children of a group are the concepts whose [`Placement::Child`] names the group. A step
/// also has a start.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Group {
    /// The name of the group. A rule ID and a finding name the group by it.
    pub name: GroupName,
    /// The concept that the start and the weighted children add up to.
    pub parent: CanonicalElementKind,
    /// Whether the group splits its parent or makes it from a start line.
    pub kind: GroupKind,
    /// `true` if the start and the weighted children equal the parent by definition. A filer
    /// reports lines outside an open group, so the children of an open group do not cover the
    /// parent.
    pub closed: bool,
}

/// The name of a group, nested by the statement of its parent.
///
/// Each statement module defines the group enum of its statement. A group can have a name before
/// the list of its statement holds it, so a placement can name it.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GroupName {
    /// A group whose parent is a balance sheet concept.
    BalanceSheet(BalanceSheetGroup),
    /// A group whose parent is an income statement concept.
    IncomeStatement(IncomeStatementGroup),
    /// A group whose parent is a cash flow concept.
    CashFlow(CashFlowGroup),
}

impl GroupName {
    /// Returns the name of the group in rule IDs and findings, such as `current-assets`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BalanceSheet(group) => group.as_str(),
            Self::IncomeStatement(group) => group.as_str(),
            Self::CashFlow(group) => group.as_str(),
        }
    }
}

impl fmt::Display for GroupName {
    /// Writes the name of the group, such as `current-assets`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The kind of a [`Group`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GroupKind {
    /// Splits the parent into its components, such as `CurrentAssets` into cash, receivables,
    /// and inventory.
    Breakdown,
    /// Makes a subtotal from a start line and items, such as `GrossProfit` from `Revenue` less
    /// `CostOfRevenue`.
    Step {
        /// The line the step starts from. It can belong to another statement.
        start: CanonicalElementKind,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_placement() {
        implements_auto_traits::<Placement>();
    }

    #[test]
    const fn should_implement_auto_traits_for_group() {
        implements_auto_traits::<Group>();
    }

    #[test]
    const fn should_implement_auto_traits_for_group_name() {
        implements_auto_traits::<GroupName>();
    }

    #[test]
    fn should_count_as_zero_if_absent_when_the_child_is_created_zero_if_absent() {
        let placement = Placement::child_zero_if_absent(
            BalanceSheetGroup::LiabilitiesAndEquity.name(),
            Weight::Plus,
        );

        let expected_result = true;

        let result = placement.is_zero_if_absent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_count_as_zero_if_absent_when_the_child_is_created_with_child() {
        let placement = Placement::child(BalanceSheetGroup::Assets.name(), Weight::Plus);

        let expected_result = false;

        let result = placement.is_zero_if_absent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_count_as_zero_if_absent_for_a_top_line() {
        let placement = Placement::Top;

        let expected_result = false;

        let result = placement.is_zero_if_absent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_kebab_case_name_for_current_assets() {
        let group = BalanceSheetGroup::CurrentAssets.name();

        let expected_result = "current-assets";

        let result = group.to_string();

        assert_eq!(result, expected_result);
    }
}
