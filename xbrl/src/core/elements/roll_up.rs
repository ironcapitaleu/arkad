//! # Roll-Up Tree
//!
//! Provides the types of the roll-up tree: the [`Placement`] of a concept, its [`Weight`], and
//! the [`Group`] that sums children into a parent.
//!
//! Each statement module holds the placements of its concepts and the groups whose parent it
//! holds. `xbrl/DESIGN.md`, "Roll-Up Tree", gives the tables.

use super::CanonicalElementKind;

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
        /// The name of the group, such as `current-assets`.
        group: &'static str,
        /// The sign with which the child adds to the parent.
        weight: Weight,
    },
}

impl Placement {
    /// Creates a [`Placement::Child`] of the group `group` with the weight `weight`.
    #[must_use]
    pub const fn child(group: &'static str, weight: Weight) -> Self {
        Self::Child { group, weight }
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
    /// The name of the group, such as `current-assets`. A rule ID and a finding name the group by
    /// it.
    pub name: &'static str,
    /// The concept that the members add up to.
    pub parent: CanonicalElementKind,
    /// Whether the group splits its parent or makes it from a start line.
    pub kind: GroupKind,
    /// `true` if the start and the weighted children equal the parent by definition. A filer
    /// reports lines outside an open group, so the children of an open group do not cover the
    /// parent.
    pub closed: bool,
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
}
