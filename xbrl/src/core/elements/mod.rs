//! # Standard Concepts
//!
//! Provides [`CanonicalElementKind`], the standard concept set, and the traits [`Concept`] and
//! [`Placed`] that describe one concept.
//!
//! The concepts nest by statement and disclosure, as the errors of `sec` nest by level:
//!
//! - [`CanonicalElementKind`] holds a [`StatementConcept`] or a [`DisclosureConcept`].
//! - [`StatementConcept`] holds a [`BalanceSheetConcept`], an [`IncomeStatementConcept`], or a
//!   [`CashFlowConcept`].
//! - [`DisclosureConcept`] holds a [`PerShareConcept`].
//!
//! Each level converts up with `From` and down with `TryFrom`. `docs/adr/0011-concept-hierarchy.md`
//! records why the concepts nest this way.
//!
//! ## Modules
//!
//! - [`disclosure`]: The concepts outside the trees of the statements.
//! - [`roll_up`]: The placements and the groups of the roll-up tree.
//! - [`statement`]: The concepts of the three statements, with their placements and groups.
//!
//! ## Usage
//!
//! ```rust
//! use xbrl::core::elements::{CanonicalElementKind, IncomeStatementConcept};
//!
//! let element = IncomeStatementConcept::NetIncome.element();
//!
//! let expected_result = Ok(IncomeStatementConcept::NetIncome);
//!
//! let result = IncomeStatementConcept::try_from(element);
//!
//! assert_eq!(result, expected_result);
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{Serialize, Serializer};

use super::invalid_value::{InvalidValue, InvalidValueReason};

#[macro_use]
mod macros;

pub mod disclosure;
pub mod roll_up;
pub mod statement;

pub use disclosure::PerShareConcept;
pub use roll_up::{Group, GroupKind, GroupName, Placement, Weight};
pub use statement::{BalanceSheetConcept, CashFlowConcept, IncomeStatementConcept};

/// A standard concept: the top of the concept hierarchy.
///
/// Silver stores the name of the innermost variant, from [`Concept::name`], such as `NetIncome`.
/// It never stores the label of `Display`, such as `Net Income`. `Serialize` writes the name, and
/// [`FromStr`] parses it back.
///
/// `Ord` sorts by statement first, then by the order of the variants in the enum of the concept.
///
/// # Examples
///
/// ```rust
/// use xbrl::core::elements::{CanonicalElementKind, IncomeStatementConcept};
///
/// let name = "NetIncome";
///
/// let expected_result = Ok(IncomeStatementConcept::NetIncome.element());
///
/// let result = name.parse::<CanonicalElementKind>();
///
/// assert_eq!(result, expected_result);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalElementKind {
    /// A concept of a statement, with a place in the roll-up tree of that statement.
    Statement(StatementConcept),
    /// A concept of a disclosure, outside the trees of the statements.
    Disclosure(DisclosureConcept),
}

/// One standard concept of an innermost enum, such as [`BalanceSheetConcept`].
pub trait Concept: Copy + Eq + 'static {
    /// Every variant, in the order of the enum.
    const ALL: &'static [Self];

    /// Returns the name in silver, such as `NetIncome`: the name of the variant.
    fn name(self) -> &'static str;

    /// Returns the label for people, such as `Net Income`.
    fn label(self) -> &'static str;
}

/// A concept with a place in the roll-up tree of a statement.
///
/// A concept of a disclosure has no placement, so its enum implements [`Concept`] only.
pub trait Placed: Concept {
    /// Returns the group and the weight of the concept, or [`Placement::Top`].
    fn placement(self) -> Placement;
}

middle_concepts! {
    /// A concept of a statement.
    pub enum StatementConcept in Statement {
        /// A concept of the balance sheet.
        BalanceSheet(BalanceSheetConcept),
        /// A concept of the income statement, comprehensive income included.
        IncomeStatement(IncomeStatementConcept),
        /// A concept of the cash flow statement.
        CashFlow(CashFlowConcept),
    }

    /// A concept of a disclosure.
    pub enum DisclosureConcept in Disclosure {
        /// A per-share amount, such as earnings per share.
        PerShare(PerShareConcept),
    }
}

impl CanonicalElementKind {
    /// Returns the name in silver of the concept, such as `NetIncome`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Statement(concept) => concept.name(),
            Self::Disclosure(concept) => concept.name(),
        }
    }

    /// Returns the label for people of the concept, such as `Net Income`.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Statement(concept) => concept.label(),
            Self::Disclosure(concept) => concept.label(),
        }
    }
}

impl fmt::Display for CanonicalElementKind {
    /// Writes the label for people, such as `Net Income`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl Serialize for CanonicalElementKind {
    /// Writes the name in silver, such as `NetIncome`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl FromStr for CanonicalElementKind {
    type Err = InvalidValue;

    /// Parses the name in silver, such as `NetIncome`, back to its concept.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidValue`] with [`InvalidValueReason::NoMatchingName`] if no concept has
    /// the input as its name. A label, such as `Net Income`, is no name.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        INNERMOST
            .iter()
            .flat_map(|elements| elements.iter())
            .copied()
            .find(|element| element.name() == input)
            .ok_or_else(|| {
                InvalidValue::new(
                    "CanonicalElementKind",
                    InvalidValueReason::NoMatchingName("standard concept"),
                    input,
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_canonical_element_kind() {
        implements_auto_traits::<CanonicalElementKind>();
    }

    #[test]
    fn should_match_innermost_when_parsing_the_name_of_every_concept_back() {
        let elements: Vec<CanonicalElementKind> = INNERMOST.concat();

        // The expected result is INNERMOST itself, the reference path: 45 written-out entries
        // would repeat the enums.
        let expected_result: Vec<Result<CanonicalElementKind, InvalidValue>> =
            elements.iter().copied().map(Ok).collect();

        let result: Vec<Result<CanonicalElementKind, InvalidValue>> = elements
            .iter()
            .map(|element| element.name().parse())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_one_distinct_name_per_concept_for_the_45_concepts_in_innermost() {
        let elements: Vec<CanonicalElementKind> = INNERMOST.concat();

        let expected_result = (45, 45);

        let result = (
            elements.len(),
            elements
                .iter()
                .map(|element| element.name())
                .collect::<BTreeSet<_>>()
                .len(),
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_name_when_no_concept_has_it() {
        let name = "Expenses";

        let expected_result = Err(InvalidValue::new(
            "CanonicalElementKind",
            InvalidValueReason::NoMatchingName("standard concept"),
            name,
        ));

        let result = name.parse::<CanonicalElementKind>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_the_label_when_it_differs_from_the_name() {
        let label = "Net Income";

        let expected_result = Err(InvalidValue::new(
            "CanonicalElementKind",
            InvalidValueReason::NoMatchingName("standard concept"),
            label,
        ));

        let result = label.parse::<CanonicalElementKind>();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_the_name_of_the_innermost_variant_for_net_income() {
        let element = IncomeStatementConcept::NetIncome.element();

        let expected_result = Some("\"NetIncome\"".to_owned());

        let result = serde_json::to_string(&element).ok();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_label_for_net_income() {
        let element = IncomeStatementConcept::NetIncome.element();

        let expected_result = "Net Income";

        let result = element.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_sort_by_statement_first_then_by_concept_when_sorting_elements() {
        let mut elements = vec![
            PerShareConcept::EarningsPerShareBasic.element(),
            CashFlowConcept::OperatingCashFlow.element(),
            IncomeStatementConcept::Revenue.element(),
            BalanceSheetConcept::RetainedEarnings.element(),
            BalanceSheetConcept::Assets.element(),
        ];

        let expected_result = vec![
            BalanceSheetConcept::Assets.element(),
            BalanceSheetConcept::RetainedEarnings.element(),
            IncomeStatementConcept::Revenue.element(),
            CashFlowConcept::OperatingCashFlow.element(),
            PerShareConcept::EarningsPerShareBasic.element(),
        ];

        elements.sort();
        let result = elements;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_equal_element_when_converting_up_one_level_at_a_time() {
        let concept = BalanceSheetConcept::Assets;

        let expected_result = concept.element();

        let result = CanonicalElementKind::from(StatementConcept::from(concept));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_back_the_element_when_converting_down_to_another_statement() {
        let element = IncomeStatementConcept::Revenue.element();

        let expected_result = Err(element);

        let result = BalanceSheetConcept::try_from(element);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_back_the_element_when_converting_a_disclosure_down_to_a_statement() {
        let element = PerShareConcept::EarningsPerShareDiluted.element();

        let expected_result = Err(element);

        let result = StatementConcept::try_from(element);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_back_the_concept_when_converting_down_to_another_innermost_enum() {
        let concept = StatementConcept::from(CashFlowConcept::DividendsPaid);

        let expected_result = Err(concept);

        let result = IncomeStatementConcept::try_from(concept);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_down_one_level_when_the_disclosure_holds_the_concept() {
        let concept = DisclosureConcept::PerShare(PerShareConcept::EarningsPerShareBasic);

        let expected_result = Ok(PerShareConcept::EarningsPerShareBasic);

        let result = PerShareConcept::try_from(concept);

        assert_eq!(result, expected_result);
    }
}
