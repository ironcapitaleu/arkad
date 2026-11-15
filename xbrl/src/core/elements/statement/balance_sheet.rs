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
        /// Total liabilities, temporary equity, and equity.
        LiabilitiesAndEquity => "Liabilities and Equity",
        /// Total liabilities.
        Liabilities => "Liabilities",
        /// Temporary equity, such as redeemable stock, including the portion of the
        /// noncontrolling interest.
        TemporaryEquity => "Temporary Equity",
        /// Stockholders' equity, including the portion of the noncontrolling interest.
        Equity => "Equity",
        /// Current liabilities subtotal.
        CurrentLiabilities => "Current Liabilities",
        /// Non-current liabilities subtotal.
        NonCurrentLiabilities => "Non-Current Liabilities",
        /// Accounts payable (current).
        AccountsPayable => "Accounts Payable",
        /// Short-term borrowings and current maturities of long-term debt, including current
        /// finance lease obligations.
        ShortTermDebt => "Short-Term Debt",
        /// Long-term debt.
        LongTermDebt => "Long-Term Debt",
        /// Stockholders' equity attributable to the parent.
        EquityAttributableToParent => "Equity Attributable to Parent",
        /// Equity of the noncontrolling interest.
        NoncontrollingInterest => "Noncontrolling Interest",
        /// Additional paid-in capital.
        AdditionalPaidInCapital => "Additional Paid-In Capital",
        /// Retained earnings (accumulated deficit).
        RetainedEarnings => "Retained Earnings",
        /// Treasury stock, at cost. The value is positive.
        TreasuryStock => "Treasury Stock",
        /// Accumulated other comprehensive income (loss), net of tax.
        AccumulatedOtherComprehensiveIncome => "Accumulated Other Comprehensive Income",
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
        /// The group whose parent is [`BalanceSheetConcept::LiabilitiesAndEquity`].
        LiabilitiesAndEquity => "liabilities-and-equity",
        /// The group whose parent is [`BalanceSheetConcept::Liabilities`].
        Liabilities => "liabilities",
        /// The group whose parent is [`BalanceSheetConcept::CurrentLiabilities`].
        CurrentLiabilities => "current-liabilities",
        /// The group whose parent is [`BalanceSheetConcept::NonCurrentLiabilities`].
        NonCurrentLiabilities => "non-current-liabilities",
        /// The group whose parent is [`BalanceSheetConcept::Equity`].
        Equity => "equity",
        /// The group whose parent is [`BalanceSheetConcept::EquityAttributableToParent`].
        EquityAttributableToParent => "equity-attributable-to-parent",
    }
}

impl Placed for BalanceSheetConcept {
    fn placement(self) -> Placement {
        match self {
            Self::Assets | Self::LiabilitiesAndEquity => Placement::Top,
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
            Self::Liabilities | Self::TemporaryEquity | Self::Equity => {
                Placement::child(BalanceSheetGroup::LiabilitiesAndEquity.name(), Weight::Plus)
            }
            Self::CurrentLiabilities | Self::NonCurrentLiabilities => {
                Placement::child(BalanceSheetGroup::Liabilities.name(), Weight::Plus)
            }
            Self::AccountsPayable | Self::ShortTermDebt => {
                Placement::child(BalanceSheetGroup::CurrentLiabilities.name(), Weight::Plus)
            }
            Self::LongTermDebt => Placement::child(
                BalanceSheetGroup::NonCurrentLiabilities.name(),
                Weight::Plus,
            ),
            Self::EquityAttributableToParent | Self::NoncontrollingInterest => {
                Placement::child(BalanceSheetGroup::Equity.name(), Weight::Plus)
            }
            Self::AdditionalPaidInCapital
            | Self::RetainedEarnings
            | Self::AccumulatedOtherComprehensiveIncome => Placement::child(
                BalanceSheetGroup::EquityAttributableToParent.name(),
                Weight::Plus,
            ),
            Self::TreasuryStock => Placement::child(
                BalanceSheetGroup::EquityAttributableToParent.name(),
                Weight::Minus,
            ),
        }
    }
}

/// The groups whose parent is a balance sheet concept, in the order in which the balance sheet
/// presents the first child of each group. `docs/adr/0013-order-of-the-groups.md` records the
/// order.
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
        name: BalanceSheetGroup::LiabilitiesAndEquity.name(),
        parent: BalanceSheetConcept::LiabilitiesAndEquity.element(),
        kind: GroupKind::Breakdown,
        closed: true,
    },
    Group {
        name: BalanceSheetGroup::Liabilities.name(),
        parent: BalanceSheetConcept::Liabilities.element(),
        kind: GroupKind::Breakdown,
        closed: true,
    },
    Group {
        name: BalanceSheetGroup::CurrentLiabilities.name(),
        parent: BalanceSheetConcept::CurrentLiabilities.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
    Group {
        name: BalanceSheetGroup::NonCurrentLiabilities.name(),
        parent: BalanceSheetConcept::NonCurrentLiabilities.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
    Group {
        name: BalanceSheetGroup::Equity.name(),
        parent: BalanceSheetConcept::Equity.element(),
        kind: GroupKind::Breakdown,
        closed: true,
    },
    Group {
        name: BalanceSheetGroup::EquityAttributableToParent.name(),
        parent: BalanceSheetConcept::EquityAttributableToParent.element(),
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

    // The test writes out the placement of every concept in one place, so a concept that moves
    // fails it. A split by group would need a filter, and a concept outside every filter would pass.
    #[allow(clippy::too_many_lines)]
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
            (BalanceSheetConcept::LiabilitiesAndEquity, Placement::Top),
            (
                BalanceSheetConcept::Liabilities,
                Placement::child(BalanceSheetGroup::LiabilitiesAndEquity.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::TemporaryEquity,
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
                BalanceSheetConcept::AccountsPayable,
                Placement::child(BalanceSheetGroup::CurrentLiabilities.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::ShortTermDebt,
                Placement::child(BalanceSheetGroup::CurrentLiabilities.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::LongTermDebt,
                Placement::child(
                    BalanceSheetGroup::NonCurrentLiabilities.name(),
                    Weight::Plus,
                ),
            ),
            (
                BalanceSheetConcept::EquityAttributableToParent,
                Placement::child(BalanceSheetGroup::Equity.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::NoncontrollingInterest,
                Placement::child(BalanceSheetGroup::Equity.name(), Weight::Plus),
            ),
            (
                BalanceSheetConcept::AdditionalPaidInCapital,
                Placement::child(
                    BalanceSheetGroup::EquityAttributableToParent.name(),
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
            (
                BalanceSheetConcept::TreasuryStock,
                Placement::child(
                    BalanceSheetGroup::EquityAttributableToParent.name(),
                    Weight::Minus,
                ),
            ),
            (
                BalanceSheetConcept::AccumulatedOtherComprehensiveIncome,
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
            "current-liabilities",
            "non-current-liabilities",
            "equity",
            "equity-attributable-to-parent",
        ];

        let result: Vec<&str> = groups.iter().map(|group| group.as_str()).collect();

        assert_eq!(result, expected_result);
    }
}
