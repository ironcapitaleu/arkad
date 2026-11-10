//! # Table Rows
//!
//! Provides the row types of the generated tables in [`super::taxonomy`].
//!
//! The rows hold `&'static str`, so the tables need no initialization at program start.

use crate::core::taxonomy::reference::{Balance, PeriodType};

/// One concept of an element schema, with its standard label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementRow {
    /// The namespace family, for example `us-gaap`, `srt`, or `dei`.
    pub family: &'static str,
    /// The local name, for example `Assets`.
    pub name: &'static str,
    /// The standard label, for example `Assets`.
    pub label: &'static str,
    /// The data type as the schema writes it, for example `xbrli:monetaryItemType`.
    pub data_type: &'static str,
    /// The balance, if the concept has one.
    pub balance: Option<Balance>,
    /// The period type.
    pub period_type: PeriodType,
}

impl ElementRow {
    /// Creates an [`ElementRow`].
    #[must_use]
    pub const fn new(
        family: &'static str,
        name: &'static str,
        label: &'static str,
        data_type: &'static str,
        balance: Option<Balance>,
        period_type: PeriodType,
    ) -> Self {
        Self {
            family,
            name,
            label,
            data_type,
            balance,
            period_type,
        }
    }
}

/// One summation arc of a calculation linkbase. Within one role, the parent is the weighted sum of
/// its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalculationRow {
    /// The role of the network of the arc, for example
    /// `http://fasb.org/us-gaap/role/statement/StatementOfIncome`.
    pub role: &'static str,
    /// The namespace family of the parent, for example `us-gaap`.
    pub parent_family: &'static str,
    /// The local name of the parent.
    pub parent_name: &'static str,
    /// The namespace family of the child. Some arcs point to `srt` concepts.
    pub child_family: &'static str,
    /// The local name of the child.
    pub child_name: &'static str,
    /// The weight as the linkbase writes it, `1.0` or `-1.0`. The plug-in parses it into the
    /// `BigDecimal` of its calculation tree.
    pub weight: &'static str,
}

impl CalculationRow {
    /// Creates a [`CalculationRow`].
    #[must_use]
    pub const fn new(
        role: &'static str,
        parent_family: &'static str,
        parent_name: &'static str,
        child_family: &'static str,
        child_name: &'static str,
        weight: &'static str,
    ) -> Self {
        Self {
            role,
            parent_family,
            parent_name,
            child_family,
            child_name,
            weight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_element_row() {
        implements_auto_traits::<ElementRow>();
    }

    #[test]
    const fn should_implement_auto_traits_for_calculation_row() {
        implements_auto_traits::<CalculationRow>();
    }
}
