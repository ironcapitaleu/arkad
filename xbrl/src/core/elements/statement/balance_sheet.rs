//! # Balance Sheet Concepts
//!
//! Provides [`BalanceSheetConcept`], its placements, and [`GROUPS`], the groups whose parent is a
//! balance sheet concept.

use crate::core::elements::Placed;
use crate::core::elements::roll_up::{Group, GroupKind, Placement, Weight};

const ASSETS: &str = "assets";
const CURRENT_ASSETS: &str = "current-assets";
const NON_CURRENT_ASSETS: &str = "non-current-assets";
const LIABILITIES_AND_EQUITY: &str = "liabilities-and-equity";
const LIABILITIES: &str = "liabilities";
const NON_CURRENT_LIABILITIES: &str = "non-current-liabilities";
const EQUITY_ATTRIBUTABLE_TO_PARENT: &str = "equity-attributable-to-parent";

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

impl Placed for BalanceSheetConcept {
    fn placement(self) -> Placement {
        match self {
            Self::Assets => Placement::Top,
            Self::CurrentAssets | Self::NonCurrentAssets => Placement::child(ASSETS, Weight::Plus),
            Self::CashAndEquivalents
            | Self::ShortTermInvestments
            | Self::AccountsReceivable
            | Self::Inventory => Placement::child(CURRENT_ASSETS, Weight::Plus),
            Self::PropertyPlantEquipment | Self::Goodwill | Self::IntangibleAssets => {
                Placement::child(NON_CURRENT_ASSETS, Weight::Plus)
            }
            Self::Liabilities | Self::Equity => {
                Placement::child(LIABILITIES_AND_EQUITY, Weight::Plus)
            }
            Self::CurrentLiabilities | Self::NonCurrentLiabilities => {
                Placement::child(LIABILITIES, Weight::Plus)
            }
            Self::LongTermDebt => Placement::child(NON_CURRENT_LIABILITIES, Weight::Plus),
            Self::RetainedEarnings => Placement::child(EQUITY_ATTRIBUTABLE_TO_PARENT, Weight::Plus),
        }
    }
}

/// The groups whose parent is a balance sheet concept, from the top of
/// the statement down.
pub const GROUPS: &[Group] = &[
    Group {
        name: ASSETS,
        parent: BalanceSheetConcept::Assets.element(),
        kind: GroupKind::Breakdown,
        closed: true,
    },
    Group {
        name: CURRENT_ASSETS,
        parent: BalanceSheetConcept::CurrentAssets.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
    Group {
        name: NON_CURRENT_ASSETS,
        parent: BalanceSheetConcept::NonCurrentAssets.element(),
        kind: GroupKind::Breakdown,
        closed: false,
    },
    Group {
        name: LIABILITIES,
        parent: BalanceSheetConcept::Liabilities.element(),
        kind: GroupKind::Breakdown,
        closed: true,
    },
    Group {
        name: NON_CURRENT_LIABILITIES,
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
                Placement::child("assets", Weight::Plus),
            ),
            (
                BalanceSheetConcept::NonCurrentAssets,
                Placement::child("assets", Weight::Plus),
            ),
            (
                BalanceSheetConcept::CashAndEquivalents,
                Placement::child("current-assets", Weight::Plus),
            ),
            (
                BalanceSheetConcept::ShortTermInvestments,
                Placement::child("current-assets", Weight::Plus),
            ),
            (
                BalanceSheetConcept::AccountsReceivable,
                Placement::child("current-assets", Weight::Plus),
            ),
            (
                BalanceSheetConcept::Inventory,
                Placement::child("current-assets", Weight::Plus),
            ),
            (
                BalanceSheetConcept::PropertyPlantEquipment,
                Placement::child("non-current-assets", Weight::Plus),
            ),
            (
                BalanceSheetConcept::Goodwill,
                Placement::child("non-current-assets", Weight::Plus),
            ),
            (
                BalanceSheetConcept::IntangibleAssets,
                Placement::child("non-current-assets", Weight::Plus),
            ),
            (
                BalanceSheetConcept::Liabilities,
                Placement::child("liabilities-and-equity", Weight::Plus),
            ),
            (
                BalanceSheetConcept::Equity,
                Placement::child("liabilities-and-equity", Weight::Plus),
            ),
            (
                BalanceSheetConcept::CurrentLiabilities,
                Placement::child("liabilities", Weight::Plus),
            ),
            (
                BalanceSheetConcept::NonCurrentLiabilities,
                Placement::child("liabilities", Weight::Plus),
            ),
            (
                BalanceSheetConcept::LongTermDebt,
                Placement::child("non-current-liabilities", Weight::Plus),
            ),
            (
                BalanceSheetConcept::RetainedEarnings,
                Placement::child("equity-attributable-to-parent", Weight::Plus),
            ),
        ];

        let result: Vec<(BalanceSheetConcept, Placement)> = concepts
            .iter()
            .map(|concept| (*concept, concept.placement()))
            .collect();

        assert_eq!(result, expected_result);
    }
}
