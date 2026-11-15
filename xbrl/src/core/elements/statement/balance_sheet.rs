//! # Balance Sheet Concepts
//!
//! Provides [`BalanceSheetConcept`] and its placements. Provides [`BalanceSheetGroup`] and
//! [`GROUPS`], the groups whose parent is a balance sheet concept.

use crate::core::elements::Placed;
use crate::core::elements::roll_up::{Group, GroupKind, Placement, Weight};

innermost_concepts! {
    /// A standard concept of the balance sheet. Each value is an instant.
    pub enum BalanceSheetConcept in Statement(StatementConcept::BalanceSheet) {
        /// Total assets.
        Assets => "Assets",
        /// Current assets subtotal.
        CurrentAssets => "Current Assets",
        /// Non-current assets subtotal.
        NonCurrentAssets => "Non-Current Assets",
        /// Cash and cash equivalents.
        CashAndEquivalents => "Cash and Equivalents",
        /// Short-term investments.
        ShortTermInvestments => "Short-Term Investments",
        /// Accounts receivable (net).
        AccountsReceivable => "Accounts Receivable",
        /// Inventory (net).
        Inventory => "Inventory",
        /// Property, plant, and equipment (net).
        PropertyPlantEquipment => "Property, Plant & Equipment",
        /// Goodwill.
        Goodwill => "Goodwill",
        /// Intangible assets (net, excluding goodwill).
        IntangibleAssets => "Intangible Assets",
        /// Total liabilities.
        Liabilities => "Liabilities",
        /// Stockholders' equity, including the portion of the noncontrolling interest.
        Equity => "Equity",
        /// Current liabilities subtotal.
        CurrentLiabilities => "Current Liabilities",
        /// Non-current liabilities subtotal.
        NonCurrentLiabilities => "Non-Current Liabilities",
        /// Long-term debt.
        LongTermDebt => "Long-Term Debt",
        /// Retained earnings (accumulated deficit).
        RetainedEarnings => "Retained Earnings",
    }
}

group_names! {
    /// A group whose parent is a balance sheet concept.
    pub enum BalanceSheetGroup in BalanceSheet {
        /// The group whose parent is [`BalanceSheetConcept::Assets`].
        Assets => "assets",
        /// The group whose parent is [`BalanceSheetConcept::CurrentAssets`].
        CurrentAssets => "current-assets",
        /// The group whose parent is [`BalanceSheetConcept::NonCurrentAssets`].
        NonCurrentAssets => "non-current-assets",
        /// The group whose parent is `LiabilitiesAndEquity`.
        LiabilitiesAndEquity => "liabilities-and-equity",
        /// The group whose parent is [`BalanceSheetConcept::Liabilities`].
        Liabilities => "liabilities",
        /// The group whose parent is [`BalanceSheetConcept::NonCurrentLiabilities`].
        NonCurrentLiabilities => "non-current-liabilities",
        /// The group whose parent is `EquityAttributableToParent`.
        EquityAttributableToParent => "equity-attributable-to-parent",
    }
}

impl Placed for BalanceSheetConcept {
    fn placement(self) -> Placement {
        match self {
            Self::Assets => Placement::Top,
            Self::CurrentAssets | Self::NonCurrentAssets => {
                Placement::child(BalanceSheetGroup::Assets.name(), Weight::Plus)
            }
            Self::CashAndEquivalents
            | Self::ShortTermInvestments
            | Self::AccountsReceivable
            | Self::Inventory => {
                Placement::child(BalanceSheetGroup::CurrentAssets.name(), Weight::Plus)
            }
            Self::PropertyPlantEquipment | Self::Goodwill | Self::IntangibleAssets => {
                Placement::child(BalanceSheetGroup::NonCurrentAssets.name(), Weight::Plus)
            }
            Self::Liabilities | Self::Equity => {
                Placement::child(BalanceSheetGroup::LiabilitiesAndEquity.name(), Weight::Plus)
            }
            Self::CurrentLiabilities | Self::NonCurrentLiabilities => {
                Placement::child(BalanceSheetGroup::Liabilities.name(), Weight::Plus)
            }
            Self::LongTermDebt => Placement::child(
                BalanceSheetGroup::NonCurrentLiabilities.name(),
                Weight::Plus,
            ),
            Self::RetainedEarnings => Placement::child(
                BalanceSheetGroup::EquityAttributableToParent.name(),
                Weight::Plus,
            ),
        }
    }
}

/// The groups whose parent is a balance sheet concept.
///
/// The list follows the order of the first row that names each group in the table of the balance
/// sheet. `docs/adr/0013-order-of-the-groups.md` records the order.
pub const GROUPS: &[Group] = &[
    Group {
        name: BalanceSheetGroup::Assets.name(),
        parent: BalanceSheetConcept::Assets.element(),
        kind: GroupKind::Breakdown,
        closed: true,
    },
    Group {
        name: BalanceSheetGroup::CurrentAssets.name(),
        parent: BalanceSheetConcept::CurrentAssets.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
    Group {
        name: BalanceSheetGroup::NonCurrentAssets.name(),
        parent: BalanceSheetConcept::NonCurrentAssets.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
    Group {
        name: BalanceSheetGroup::Liabilities.name(),
        parent: BalanceSheetConcept::Liabilities.element(),
        kind: GroupKind::Breakdown,
        closed: true,
    },
    Group {
        name: BalanceSheetGroup::NonCurrentLiabilities.name(),
        parent: BalanceSheetConcept::NonCurrentLiabilities.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
];

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::Concept;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn should_implement_auto_traits_for_balance_sheet_concept() {
        implements_auto_traits::<BalanceSheetConcept>();
    }

    #[test]
    fn should_place_every_balance_sheet_concept_as_the_table_gives_it() {
        let concepts = BalanceSheetConcept::ALL;

        let expected_result = vec![
            (BalanceSheetConcept::Assets, Placement::Top),
            (
                BalanceSheetConcept::CurrentAssets,
                Placement::child(BalanceSheetGroup::Assets.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::NonCurrentAssets,
                Placement::child(BalanceSheetGroup::Assets.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::CashAndEquivalents,
                Placement::child(BalanceSheetGroup::CurrentAssets.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::ShortTermInvestments,
                Placement::child(BalanceSheetGroup::CurrentAssets.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::AccountsReceivable,
                Placement::child(BalanceSheetGroup::CurrentAssets.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::Inventory,
                Placement::child(BalanceSheetGroup::CurrentAssets.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::PropertyPlantEquipment,
                Placement::child(BalanceSheetGroup::NonCurrentAssets.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::Goodwill,
                Placement::child(BalanceSheetGroup::NonCurrentAssets.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::IntangibleAssets,
                Placement::child(BalanceSheetGroup::NonCurrentAssets.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::Liabilities,
                Placement::child(BalanceSheetGroup::LiabilitiesAndEquity.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::Equity,
                Placement::child(BalanceSheetGroup::LiabilitiesAndEquity.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::CurrentLiabilities,
                Placement::child(BalanceSheetGroup::Liabilities.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::NonCurrentLiabilities,
                Placement::child(BalanceSheetGroup::Liabilities.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::LongTermDebt,
                Placement::child(
                    BalanceSheetGroup::NonCurrentLiabilities.name(),
                    Weight::Plus,
                ),
            ),
            (
                BalanceSheetConcept::RetainedEarnings,
                Placement::child(
                    BalanceSheetGroup::EquityAttributableToParent.name(),
                    Weight::Plus,
                ),
            ),
        ];

        let result: Vec<(BalanceSheetConcept, Placement)> = concepts
            .iter()
            .map(|concept| (*concept, concept.placement()))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_the_kebab_case_name_for_every_balance_sheet_group() {
        let groups = BalanceSheetGroup::ALL;

        let expected_result = vec![
            "assets",
            "current-assets",
            "non-current-assets",
            "liabilities-and-equity",
            "liabilities",
            "non-current-liabilities",
            "equity-attributable-to-parent",
        ];

        let result: Vec<&str> = groups.iter().map(|group| group.as_str()).collect();

        assert_eq!(result, expected_result);
    }
}
