//! # US-GAAP Concept Mappings
//!
//! Canonical name constants and concept definition arrays mapping
//! XBRL taxonomy concept names to canonical financial elements.
//!
//! Which concepts a list can hold: `xbrl/DESIGN.md`, "`us_gaap`".
//!
//! Each definition names the ID of its alias rule. The rule name is the kebab case of its
//! canonical element, as `xbrl/DESIGN.md`, "`us_gaap`", describes.
//!
//! ## Two-Tier Validation
//! - **Required**: Structural backbone concepts every public company reports.
//! - **Optional**: Enriching detail extracted when available, skipped when absent.

use crate::core::elements::{
    BalanceSheetConcept, CanonicalElementKind, CashFlowConcept, IncomeStatementConcept,
    PerShareConcept,
};

/// A mapping from one or more XBRL concept names to a canonical financial element.
///
/// The `xbrl_keys` array is ordered by priority: the first key that matches
/// in the SEC JSON wins. This handles taxonomy version changes over time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConceptDefinition {
    /// The canonical element this concept maps to.
    canonical_element: CanonicalElementKind,
    /// XBRL concept names to search for, in priority order.
    xbrl_keys: &'static [&'static str],
    /// The ID of the alias rule, for example `us-gaap/alias/revenue@1`.
    alias_rule: &'static str,
    /// Whether this concept is required (validation fails if absent).
    required: bool,
}

impl ConceptDefinition {
    /// Creates a new [`ConceptDefinition`].
    #[must_use]
    pub const fn new(
        canonical_element: CanonicalElementKind,
        xbrl_keys: &'static [&'static str],
        alias_rule: &'static str,
        required: bool,
    ) -> Self {
        Self {
            canonical_element,
            xbrl_keys,
            alias_rule,
            required,
        }
    }

    /// Returns the canonical element this definition maps to.
    #[must_use]
    pub const fn canonical_element(&self) -> CanonicalElementKind {
        self.canonical_element
    }

    /// Returns the XBRL concept keys in priority order.
    #[must_use]
    pub const fn xbrl_keys(&self) -> &'static [&'static str] {
        self.xbrl_keys
    }

    /// Returns the ID of the alias rule, for example `us-gaap/alias/revenue@1`.
    #[must_use]
    pub const fn alias_rule(&self) -> &'static str {
        self.alias_rule
    }

    /// Returns `true` if this concept is required for valid output.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

/// The parts of a standard concept that stand for the whole when a filing reports no other part.
///
/// A part counts only if no concept of the alias list resolves, and only if the filing reports
/// none of the other parts in the period. `xbrl/DESIGN.md`, "`us_gaap`", gives the rule.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartDefinition {
    /// The standard concept the parts stand for.
    canonical_element: CanonicalElementKind,
    /// The parts, in priority order.
    parts: &'static [&'static str],
    /// The other parts that count alone, without the concepts below them.
    other_parts: &'static [&'static str],
    /// The other parts that also count with each concept below them in the 2026 calculation tree.
    other_part_trees: &'static [&'static str],
    /// The ID of the part rule, for example `us-gaap/part/equity@1`.
    part_rule: &'static str,
}

impl PartDefinition {
    /// Creates a new [`PartDefinition`].
    #[must_use]
    pub const fn new(
        canonical_element: CanonicalElementKind,
        parts: &'static [&'static str],
        other_parts: &'static [&'static str],
        other_part_trees: &'static [&'static str],
        part_rule: &'static str,
    ) -> Self {
        Self {
            canonical_element,
            parts,
            other_parts,
            other_part_trees,
            part_rule,
        }
    }

    /// Returns the standard concept the parts stand for.
    #[must_use]
    pub const fn canonical_element(&self) -> CanonicalElementKind {
        self.canonical_element
    }

    /// Returns the parts in priority order.
    #[must_use]
    pub const fn parts(&self) -> &'static [&'static str] {
        self.parts
    }

    /// Returns the other parts that count alone, without the concepts below them.
    #[must_use]
    pub const fn other_parts(&self) -> &'static [&'static str] {
        self.other_parts
    }

    /// Returns the other parts that also count with each concept below them in the 2026
    /// calculation tree.
    #[must_use]
    pub const fn other_part_trees(&self) -> &'static [&'static str] {
        self.other_part_trees
    }

    /// Returns the ID of the part rule, for example `us-gaap/part/equity@1`.
    #[must_use]
    pub const fn part_rule(&self) -> &'static str {
        self.part_rule
    }
}

/// The ID of the alias rule of [`IncomeStatementConcept::Revenue`].
pub const REVENUE_RULE: &str = "us-gaap/alias/revenue@1";
/// The ID of the alias rule of [`IncomeStatementConcept::OperatingIncome`].
pub const OPERATING_INCOME_RULE: &str = "us-gaap/alias/operating-income@1";
/// The ID of the alias rule of [`IncomeStatementConcept::NetIncome`].
pub const NET_INCOME_RULE: &str = "us-gaap/alias/net-income@1";
/// The ID of the alias rule of [`BalanceSheetConcept::Assets`].
pub const ASSETS_RULE: &str = "us-gaap/alias/assets@1";
/// The ID of the alias rule of [`BalanceSheetConcept::Liabilities`].
pub const LIABILITIES_RULE: &str = "us-gaap/alias/liabilities@1";
/// The ID of the alias rule of [`BalanceSheetConcept::Equity`].
pub const EQUITY_RULE: &str = "us-gaap/alias/equity@1";
/// The ID of the alias rule of [`CashFlowConcept::OperatingCashFlow`].
pub const OPERATING_CASH_FLOW_RULE: &str = "us-gaap/alias/operating-cash-flow@1";
/// The ID of the alias rule of [`CashFlowConcept::InvestingCashFlow`].
pub const INVESTING_CASH_FLOW_RULE: &str = "us-gaap/alias/investing-cash-flow@1";
/// The ID of the alias rule of [`CashFlowConcept::FinancingCashFlow`].
pub const FINANCING_CASH_FLOW_RULE: &str = "us-gaap/alias/financing-cash-flow@1";

/// Required concept definitions that every public company is expected to report.
/// Validation fails if any of these cannot be resolved in the SEC response.
pub const REQUIRED_CONCEPTS: &[ConceptDefinition] = &[
    // Income Statement backbone
    ConceptDefinition::new(
        IncomeStatementConcept::Revenue.element(),
        // The two `RevenueFromContractWithCustomer` concepts are parts, next to
        // `RevenueNotFromContractWithCustomer`. `PART_CONCEPTS` holds them.
        &["Revenues", "SalesRevenueNet"],
        REVENUE_RULE,
        true,
    ),
    ConceptDefinition::new(
        IncomeStatementConcept::OperatingIncome.element(),
        &["OperatingIncomeLoss", "IncomeLossFromOperations"],
        OPERATING_INCOME_RULE,
        true,
    ),
    ConceptDefinition::new(
        IncomeStatementConcept::NetIncome.element(),
        // `ProfitLoss` holds the noncontrolling interest, as the equity of the `Equity` list does.
        // `NetIncomeLoss` is the part of the parent. `PART_CONCEPTS` holds it.
        &["ProfitLoss"],
        NET_INCOME_RULE,
        true,
    ),
    // Balance Sheet backbone
    ConceptDefinition::new(
        BalanceSheetConcept::Assets.element(),
        &["Assets"],
        ASSETS_RULE,
        true,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::Liabilities.element(),
        &["Liabilities"],
        LIABILITIES_RULE,
        true,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::Equity.element(),
        // The total with the noncontrolling interest is the equity of
        // `Assets = Liabilities + Equity`. `StockholdersEquity` is the part of the parent.
        // `PART_CONCEPTS` holds it.
        &["StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest"],
        EQUITY_RULE,
        true,
    ),
    // Cash Flow backbone
    ConceptDefinition::new(
        CashFlowConcept::OperatingCashFlow.element(),
        &["NetCashProvidedByUsedInOperatingActivities"],
        OPERATING_CASH_FLOW_RULE,
        true,
    ),
    ConceptDefinition::new(
        CashFlowConcept::InvestingCashFlow.element(),
        &["NetCashProvidedByUsedInInvestingActivities"],
        INVESTING_CASH_FLOW_RULE,
        true,
    ),
    ConceptDefinition::new(
        CashFlowConcept::FinancingCashFlow.element(),
        &["NetCashProvidedByUsedInFinancingActivities"],
        FINANCING_CASH_FLOW_RULE,
        true,
    ),
];

/// The ID of the alias rule of [`BalanceSheetConcept::CurrentAssets`].
pub const CURRENT_ASSETS_RULE: &str = "us-gaap/alias/current-assets@1";
/// The ID of the alias rule of [`BalanceSheetConcept::NonCurrentAssets`].
pub const NON_CURRENT_ASSETS_RULE: &str = "us-gaap/alias/non-current-assets@1";
/// The ID of the alias rule of [`BalanceSheetConcept::CashAndEquivalents`].
pub const CASH_AND_EQUIVALENTS_RULE: &str = "us-gaap/alias/cash-and-equivalents@1";
/// The ID of the alias rule of [`BalanceSheetConcept::ShortTermInvestments`].
pub const SHORT_TERM_INVESTMENTS_RULE: &str = "us-gaap/alias/short-term-investments@1";
/// The ID of the alias rule of [`BalanceSheetConcept::AccountsReceivable`].
pub const ACCOUNTS_RECEIVABLE_RULE: &str = "us-gaap/alias/accounts-receivable@1";
/// The ID of the alias rule of [`BalanceSheetConcept::Inventory`].
pub const INVENTORY_RULE: &str = "us-gaap/alias/inventory@1";
/// The ID of the alias rule of [`BalanceSheetConcept::PropertyPlantEquipment`].
pub const PROPERTY_PLANT_EQUIPMENT_RULE: &str = "us-gaap/alias/property-plant-equipment@1";
/// The ID of the alias rule of [`BalanceSheetConcept::Goodwill`].
pub const GOODWILL_RULE: &str = "us-gaap/alias/goodwill@1";
/// The ID of the alias rule of [`BalanceSheetConcept::IntangibleAssets`].
pub const INTANGIBLE_ASSETS_RULE: &str = "us-gaap/alias/intangible-assets@1";
/// The ID of the alias rule of [`BalanceSheetConcept::CurrentLiabilities`].
pub const CURRENT_LIABILITIES_RULE: &str = "us-gaap/alias/current-liabilities@1";
/// The ID of the alias rule of [`BalanceSheetConcept::NonCurrentLiabilities`].
pub const NON_CURRENT_LIABILITIES_RULE: &str = "us-gaap/alias/non-current-liabilities@1";
/// The ID of the alias rule of [`BalanceSheetConcept::LongTermDebt`].
pub const LONG_TERM_DEBT_RULE: &str = "us-gaap/alias/long-term-debt@1";
/// The ID of the alias rule of [`BalanceSheetConcept::RetainedEarnings`].
pub const RETAINED_EARNINGS_RULE: &str = "us-gaap/alias/retained-earnings@1";
/// The ID of the alias rule of [`IncomeStatementConcept::CostOfRevenue`].
pub const COST_OF_REVENUE_RULE: &str = "us-gaap/alias/cost-of-revenue@1";
/// The ID of the alias rule of [`IncomeStatementConcept::GrossProfit`].
pub const GROSS_PROFIT_RULE: &str = "us-gaap/alias/gross-profit@1";
/// The ID of the alias rule of [`IncomeStatementConcept::SellingGeneralAdmin`].
pub const SELLING_GENERAL_ADMIN_RULE: &str = "us-gaap/alias/selling-general-admin@1";
/// The ID of the alias rule of [`IncomeStatementConcept::ResearchDevelopment`].
pub const RESEARCH_DEVELOPMENT_RULE: &str = "us-gaap/alias/research-development@1";
/// The ID of the alias rule of [`IncomeStatementConcept::InterestExpense`].
pub const INTEREST_EXPENSE_RULE: &str = "us-gaap/alias/interest-expense@1";
/// The ID of the alias rule of [`PerShareConcept::EarningsPerShareBasic`].
pub const EARNINGS_PER_SHARE_BASIC_RULE: &str = "us-gaap/alias/earnings-per-share-basic@1";
/// The ID of the alias rule of [`PerShareConcept::EarningsPerShareDiluted`].
pub const EARNINGS_PER_SHARE_DILUTED_RULE: &str = "us-gaap/alias/earnings-per-share-diluted@1";
/// The ID of the alias rule of [`CashFlowConcept::DepreciationAmortization`].
pub const DEPRECIATION_AMORTIZATION_RULE: &str = "us-gaap/alias/depreciation-amortization@1";
/// The ID of the alias rule of [`CashFlowConcept::CapitalExpenditures`].
pub const CAPITAL_EXPENDITURES_RULE: &str = "us-gaap/alias/capital-expenditures@1";
/// The ID of the alias rule of [`CashFlowConcept::DividendsPaid`].
pub const DIVIDENDS_PAID_RULE: &str = "us-gaap/alias/dividends-paid@1";
/// The ID of the alias rule of [`CashFlowConcept::ShareRepurchases`].
pub const SHARE_REPURCHASES_RULE: &str = "us-gaap/alias/share-repurchases@1";
/// The ID of the alias rule of [`CashFlowConcept::ShareBasedCompensation`].
pub const SHARE_BASED_COMPENSATION_RULE: &str = "us-gaap/alias/share-based-compensation@1";
/// The ID of the alias rule of [`IncomeStatementConcept::OtherComprehensiveIncome`].
pub const OTHER_COMPREHENSIVE_INCOME_RULE: &str = "us-gaap/alias/other-comprehensive-income@1";
/// The ID of the alias rule of [`IncomeStatementConcept::ComprehensiveIncome`].
pub const COMPREHENSIVE_INCOME_RULE: &str = "us-gaap/alias/comprehensive-income@1";

/// Optional concept definitions that enrich financial statements when available.
/// Their absence does not cause validation failure.
pub const OPTIONAL_CONCEPTS: &[ConceptDefinition] = &[
    // Balance Sheet detail
    ConceptDefinition::new(
        BalanceSheetConcept::CurrentAssets.element(),
        &["AssetsCurrent"],
        CURRENT_ASSETS_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::NonCurrentAssets.element(),
        // Most filers report no noncurrent total. Tier 4, which is not built yet, sums the
        // children of `AssetsNoncurrent` along the calculation tree only if the filing reports
        // every child. So most filings give no value for this concept.
        &["AssetsNoncurrent"],
        NON_CURRENT_ASSETS_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::CashAndEquivalents.element(),
        // `Cash` is a part, next to the cash equivalents. `PART_CONCEPTS` holds it.
        &["CashAndCashEquivalentsAtCarryingValue"],
        CASH_AND_EQUIVALENTS_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::ShortTermInvestments.element(),
        // `MarketableSecuritiesCurrent` is a part, next to debt securities, equity securities,
        // and other short-term investments. `PART_CONCEPTS` holds it.
        &["ShortTermInvestments"],
        SHORT_TERM_INVESTMENTS_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::AccountsReceivable.element(),
        // `ReceivablesNetCurrent` is not in the list. It is wider than the standard concept: it
        // also holds receivables that are not from customers, such as notes and receivables from
        // related parties. A filer that shows one line for all receivables gets no value.
        &["AccountsReceivableNetCurrent"],
        ACCOUNTS_RECEIVABLE_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::Inventory.element(),
        &["InventoryNet"],
        INVENTORY_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::PropertyPlantEquipment.element(),
        // `PropertyPlantAndEquipmentAndFinanceLeaseRightOfUseAssetAfterAccumulatedDepreciationAndAmortization`
        // is not in the list. It is wider than the standard concept: it also holds the right-of-use
        // assets of finance leases. A filer that shows both in one line gets no value.
        &["PropertyPlantAndEquipmentNet"],
        PROPERTY_PLANT_EQUIPMENT_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::Goodwill.element(),
        &["Goodwill"],
        GOODWILL_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::IntangibleAssets.element(),
        // `FiniteLivedIntangibleAssetsNet` is a part, next to
        // `IndefiniteLivedIntangibleAssetsExcludingGoodwill`. `PART_CONCEPTS` holds it.
        &["IntangibleAssetsNetExcludingGoodwill"],
        INTANGIBLE_ASSETS_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::CurrentLiabilities.element(),
        &["LiabilitiesCurrent"],
        CURRENT_LIABILITIES_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::NonCurrentLiabilities.element(),
        // Most filers report no noncurrent total. Tier 4, which is not built yet, sums the
        // children of `LiabilitiesNoncurrent` along the calculation tree only if the filing
        // reports every child. So most filings give no value for this concept.
        &["LiabilitiesNoncurrent"],
        NON_CURRENT_LIABILITIES_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::LongTermDebt.element(),
        // The parent is `NonCurrentLiabilities`, so the list holds the noncurrent part only.
        // Two concepts are not in the list, because they are wider than the standard concept.
        // `LongTermDebt` also holds the current maturities. `LongTermDebtAndCapitalLeaseObligations`
        // also holds finance lease obligations. A filer that shows debt and finance leases in one
        // line gets no value.
        &["LongTermDebtNoncurrent"],
        LONG_TERM_DEBT_RULE,
        false,
    ),
    ConceptDefinition::new(
        BalanceSheetConcept::RetainedEarnings.element(),
        &["RetainedEarningsAccumulatedDeficit"],
        RETAINED_EARNINGS_RULE,
        false,
    ),
    // Income Statement detail
    ConceptDefinition::new(
        IncomeStatementConcept::CostOfRevenue.element(),
        // `CostOfGoodsAndServicesSold` is a part. `PART_CONCEPTS` holds it.
        &["CostOfRevenue"],
        COST_OF_REVENUE_RULE,
        false,
    ),
    ConceptDefinition::new(
        IncomeStatementConcept::GrossProfit.element(),
        &["GrossProfit"],
        GROSS_PROFIT_RULE,
        false,
    ),
    ConceptDefinition::new(
        IncomeStatementConcept::SellingGeneralAdmin.element(),
        // `GeneralAndAdministrativeExpense` is a part. `PART_CONCEPTS` holds it.
        &["SellingGeneralAndAdministrativeExpense"],
        SELLING_GENERAL_ADMIN_RULE,
        false,
    ),
    ConceptDefinition::new(
        IncomeStatementConcept::ResearchDevelopment.element(),
        // `ResearchAndDevelopmentExpenseExcludingAcquiredInProcessCost` is a part.
        // `PART_CONCEPTS` holds it.
        &["ResearchAndDevelopmentExpense"],
        RESEARCH_DEVELOPMENT_RULE,
        false,
    ),
    ConceptDefinition::new(
        IncomeStatementConcept::InterestExpense.element(),
        // `InterestExpenseNonoperating` is a part. `PART_CONCEPTS` holds it.
        &["InterestExpense"],
        INTEREST_EXPENSE_RULE,
        false,
    ),
    ConceptDefinition::new(
        PerShareConcept::EarningsPerShareBasic.element(),
        // `EarningsPerShareBasicAndDiluted` is one value for both, so it equals the basic value.
        // The 2026 schema no longer holds it, but older filings report it.
        &["EarningsPerShareBasic", "EarningsPerShareBasicAndDiluted"],
        EARNINGS_PER_SHARE_BASIC_RULE,
        false,
    ),
    ConceptDefinition::new(
        PerShareConcept::EarningsPerShareDiluted.element(),
        // `EarningsPerShareBasicAndDiluted` is one value for both, so it equals the diluted value.
        // The 2026 schema no longer holds it, but older filings report it.
        &["EarningsPerShareDiluted", "EarningsPerShareBasicAndDiluted"],
        EARNINGS_PER_SHARE_DILUTED_RULE,
        false,
    ),
    // Cash Flow detail
    ConceptDefinition::new(
        CashFlowConcept::DepreciationAmortization.element(),
        // The add-back of the cash flow statement. `DepreciationAndAmortization` of the income
        // statement is not in the list, because it can leave out the part in cost of revenue.
        // `DepreciationAmortizationAndAccretionNet` is wider: it also holds accretion.
        // `Depreciation` is a part. `PART_CONCEPTS` holds it.
        &["DepreciationDepletionAndAmortization"],
        DEPRECIATION_AMORTIZATION_RULE,
        false,
    ),
    ConceptDefinition::new(
        CashFlowConcept::CapitalExpenditures.element(),
        // The payments for every productive asset, intangible assets and software included.
        // `PaymentsToAcquirePropertyPlantAndEquipment` is a part. `PART_CONCEPTS` holds it.
        &["PaymentsToAcquireProductiveAssets"],
        CAPITAL_EXPENDITURES_RULE,
        false,
    ),
    ConceptDefinition::new(
        CashFlowConcept::DividendsPaid.element(),
        // `PaymentsOfDividendsCommonStock` is a part. `PART_CONCEPTS` holds it.
        &["PaymentsOfDividends"],
        DIVIDENDS_PAID_RULE,
        false,
    ),
    ConceptDefinition::new(
        CashFlowConcept::ShareRepurchases.element(),
        // `PaymentsForRepurchaseOfCommonStock` is a part. `PART_CONCEPTS` holds it.
        &["PaymentsForRepurchaseOfEquity"],
        SHARE_REPURCHASES_RULE,
        false,
    ),
    ConceptDefinition::new(
        CashFlowConcept::ShareBasedCompensation.element(),
        // The add-back of the cash flow statement. `AllocatedShareBasedCompensationExpense` is
        // not in the list, because it differs when a filer capitalizes part of the cost.
        &["ShareBasedCompensation"],
        SHARE_BASED_COMPENSATION_RULE,
        false,
    ),
    // Comprehensive income. Many filers present no other comprehensive income, so these lists
    // are optional.
    ConceptDefinition::new(
        IncomeStatementConcept::OtherComprehensiveIncome.element(),
        // The total with the noncontrolling interest, as `NetIncome` and `Equity` hold it.
        // `OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToParent` is a part.
        // `PART_CONCEPTS` holds it.
        &["OtherComprehensiveIncomeLossNetOfTax"],
        OTHER_COMPREHENSIVE_INCOME_RULE,
        false,
    ),
    ConceptDefinition::new(
        IncomeStatementConcept::ComprehensiveIncome.element(),
        // The total with the noncontrolling interest, as `NetIncome` and `Equity` hold it.
        // `ComprehensiveIncomeNetOfTax` is the part of the parent. `PART_CONCEPTS` holds it.
        &["ComprehensiveIncomeNetOfTaxIncludingPortionAttributableToNoncontrollingInterest"],
        COMPREHENSIVE_INCOME_RULE,
        false,
    ),
];

/// The ID of the part rule of [`IncomeStatementConcept::Revenue`].
pub const REVENUE_PART_RULE: &str = "us-gaap/part/revenue@1";
/// The ID of the part rule of [`IncomeStatementConcept::NetIncome`].
pub const NET_INCOME_PART_RULE: &str = "us-gaap/part/net-income@1";
/// The ID of the part rule of [`BalanceSheetConcept::Equity`].
pub const EQUITY_PART_RULE: &str = "us-gaap/part/equity@1";
/// The ID of the part rule of [`BalanceSheetConcept::CashAndEquivalents`].
pub const CASH_AND_EQUIVALENTS_PART_RULE: &str = "us-gaap/part/cash-and-equivalents@1";
/// The ID of the part rule of [`BalanceSheetConcept::ShortTermInvestments`].
pub const SHORT_TERM_INVESTMENTS_PART_RULE: &str = "us-gaap/part/short-term-investments@1";
/// The ID of the part rule of [`BalanceSheetConcept::IntangibleAssets`].
pub const INTANGIBLE_ASSETS_PART_RULE: &str = "us-gaap/part/intangible-assets@1";
/// The ID of the part rule of [`IncomeStatementConcept::CostOfRevenue`].
pub const COST_OF_REVENUE_PART_RULE: &str = "us-gaap/part/cost-of-revenue@1";
/// The ID of the part rule of [`IncomeStatementConcept::SellingGeneralAdmin`].
pub const SELLING_GENERAL_ADMIN_PART_RULE: &str = "us-gaap/part/selling-general-admin@1";
/// The ID of the part rule of [`IncomeStatementConcept::ResearchDevelopment`].
pub const RESEARCH_DEVELOPMENT_PART_RULE: &str = "us-gaap/part/research-development@1";
/// The ID of the part rule of [`IncomeStatementConcept::InterestExpense`].
pub const INTEREST_EXPENSE_PART_RULE: &str = "us-gaap/part/interest-expense@1";
/// The ID of the part rule of [`CashFlowConcept::DepreciationAmortization`].
pub const DEPRECIATION_AMORTIZATION_PART_RULE: &str = "us-gaap/part/depreciation-amortization@1";
/// The ID of the part rule of [`CashFlowConcept::CapitalExpenditures`].
pub const CAPITAL_EXPENDITURES_PART_RULE: &str = "us-gaap/part/capital-expenditures@1";
/// The ID of the part rule of [`CashFlowConcept::DividendsPaid`].
pub const DIVIDENDS_PAID_PART_RULE: &str = "us-gaap/part/dividends-paid@1";
/// The ID of the part rule of [`CashFlowConcept::ShareRepurchases`].
pub const SHARE_REPURCHASES_PART_RULE: &str = "us-gaap/part/share-repurchases@1";
/// The ID of the part rule of [`IncomeStatementConcept::OtherComprehensiveIncome`].
pub const OTHER_COMPREHENSIVE_INCOME_PART_RULE: &str = "us-gaap/part/other-comprehensive-income@1";
/// The ID of the part rule of [`IncomeStatementConcept::ComprehensiveIncome`].
pub const COMPREHENSIVE_INCOME_PART_RULE: &str = "us-gaap/part/comprehensive-income@1";

/// The parts of the standard concepts. Each element has an alias list in [`REQUIRED_CONCEPTS`] or
/// [`OPTIONAL_CONCEPTS`].
///
/// The other parts are the concepts that sum to the total with the part: the siblings of the part,
/// and the siblings of each concept between the part and the total, in the 2026 calculation tree.
/// An other part in `other_part_trees` also counts with each concept below it, because filers
/// often report the concepts below it instead. A sibling that filers also report outside the
/// total is left out, because its fact does not show a second part of the total. A comment names
/// each sibling that is left out.
pub const PART_CONCEPTS: &[PartDefinition] = &[
    PartDefinition::new(
        IncomeStatementConcept::Revenue.element(),
        &[
            "RevenueFromContractWithCustomerExcludingAssessedTax",
            "RevenueFromContractWithCustomerIncludingAssessedTax",
        ],
        // The revenue lines below `Revenues` of banks, insurers, lessors, and servicers. Left out,
        // because filers also report them outside revenue: `GainsLossesOnSalesOfAssets`,
        // `OtherIncome`, `OtherOperatingIncome`, `RealizedInvestmentGainsLosses`, and
        // `SubleaseIncome`. In two roles, `NoninterestIncome` is the parent of a part. Of the
        // concepts below `NoninterestIncome`, `NetInvestmentIncome`, and
        // `InterestIncomeExpenseAfterProvisionForLoanLoss`, only the sibling of both parts counts.
        // The rest holds interest expense and gains that most filers report outside revenue.
        &[
            "ContractuallySpecifiedServicingFeesLateFeesAndAncillaryFeesEarnedInExchangeForServicingFinancialAssets",
            "DirectFinancingLeaseInterestIncome",
            "DirectFinancingLeaseRevenue",
            "DirectFinancingLeaseVariableLeaseIncome",
            "FeeIncome",
            "InsuranceAgencyManagementFee",
            "InsuranceCommissionsAndFees",
            "InterestIncomeExpenseAfterProvisionForLoanLoss",
            "NetInvestmentIncome",
            "NoninterestIncome",
            "RevenueNotFromContractWithCustomer",
            "RevenueNotFromContractWithCustomerExcludingInterestIncome",
            "RevenuesExcludingInterestAndDividends",
            "SaleOfTrustAssetsToPayExpenses",
            "SalesTypeLeaseInterestIncome",
            "SalesTypeLeaseRevenue",
            "SalesTypeLeaseVariableLeaseIncome",
        ],
        &["OperatingLeaseLeaseIncome", "PremiumsEarnedNet"],
        REVENUE_PART_RULE,
    ),
    PartDefinition::new(
        IncomeStatementConcept::NetIncome.element(),
        &["NetIncomeLoss"],
        // `NetIncomeLoss` is `ProfitLoss` minus the share of the noncontrolling interest.
        // `ProfitLoss` is the total. As an other part it blocks the part if the filing reports the
        // total only with a member, such as the noncontrolling interest.
        &["ProfitLoss"],
        &["NetIncomeLossAttributableToNoncontrollingInterest"],
        NET_INCOME_PART_RULE,
    ),
    PartDefinition::new(
        BalanceSheetConcept::Equity.element(),
        &["StockholdersEquity"],
        // The total blocks the part if the filing reports it only with a member, such as the
        // noncontrolling interest on the equity components axis.
        &["StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest"],
        &["MinorityInterest"],
        EQUITY_PART_RULE,
    ),
    PartDefinition::new(
        BalanceSheetConcept::CashAndEquivalents.element(),
        &["Cash"],
        // `Cash` sits below `CashAndDueFromBanks`, next to `DueFromBanks`. The cash equivalents
        // and the deposits in banks sit one level up.
        &["DueFromBanks", "InterestBearingDepositsInBanks"],
        &["CashEquivalentsAtCarryingValue"],
        CASH_AND_EQUIVALENTS_PART_RULE,
    ),
    PartDefinition::new(
        BalanceSheetConcept::ShortTermInvestments.element(),
        &["MarketableSecuritiesCurrent"],
        &["EquitySecuritiesFvNi", "OtherShortTermInvestments"],
        &["DebtSecuritiesCurrent"],
        SHORT_TERM_INVESTMENTS_PART_RULE,
    ),
    PartDefinition::new(
        BalanceSheetConcept::IntangibleAssets.element(),
        &["FiniteLivedIntangibleAssetsNet"],
        // `CapitalizedComputerSoftwareNet` is a sibling of the part, but it is left out, because
        // filers also report internal-use software in property, plant, and equipment.
        &[],
        &["IndefiniteLivedIntangibleAssetsExcludingGoodwill"],
        INTANGIBLE_ASSETS_PART_RULE,
    ),
    PartDefinition::new(
        IncomeStatementConcept::CostOfRevenue.element(),
        &["CostOfGoodsAndServicesSold"],
        // The cost lines below `CostOfRevenue` of insurers, lenders, and servicers. Left out:
        // `ProvisionForLoanLeaseAndOtherLosses`, because banks report it outside cost of revenue.
        &[
            "AmortizationOfMortgageServicingRightsMSRs",
            "AmortizationOfValueOfBusinessAcquiredVOBA",
            "DeferredPolicyAcquisitionCostAmortizationExpense",
            "DeferredSalesInducementsAmortizationExpense",
            "FinancingInterestExpense",
            "InterestCreditedToPolicyholdersAccountBalances",
            "MerchantMarineOperatingDifferentialSubsidy",
            "OtherCostOfOperatingRevenue",
            "PolicyholderBenefitsAndClaimsIncurredNet",
            "PolicyholderDividends",
            "PresentValueOfFutureInsuranceProfitsAmortizationExpense1",
        ],
        &[],
        COST_OF_REVENUE_PART_RULE,
    ),
    PartDefinition::new(
        IncomeStatementConcept::SellingGeneralAdmin.element(),
        &["GeneralAndAdministrativeExpense"],
        // A filer with no selling expense reports only `GeneralAndAdministrativeExpense`. Most
        // concepts below `SellingAndMarketingExpense` do not count, because filers disclose
        // advertising and warranty expense in the notes, whatever line holds it.
        // `SellingExpense` and `MarketingExpense` count, because filers report them as a line of
        // the statement.
        &[
            "MarketingExpense",
            "OtherSellingGeneralAndAdministrativeExpense",
            "SellingAndMarketingExpense",
            "SellingExpense",
        ],
        &[],
        SELLING_GENERAL_ADMIN_PART_RULE,
    ),
    PartDefinition::new(
        IncomeStatementConcept::ResearchDevelopment.element(),
        &["ResearchAndDevelopmentExpenseExcludingAcquiredInProcessCost"],
        &[
            "ResearchAndDevelopmentAssetAcquiredOtherThanThroughBusinessCombinationWrittenOff",
            "ResearchAndDevelopmentExpenseSoftwareExcludingAcquiredInProcessCost",
        ],
        &[],
        RESEARCH_DEVELOPMENT_PART_RULE,
    ),
    PartDefinition::new(
        IncomeStatementConcept::InterestExpense.element(),
        &["InterestExpenseNonoperating"],
        // The 2026 calculation tree has no arc from `InterestExpense` to these. Its label,
        // "Interest Expense, Operating and Nonoperating", names the two parts. A commercial filer
        // reports its interest expense as `InterestExpenseNonoperating`. A bank reports
        // `InterestExpenseOperating`, or the concepts below it, and it gets a value only from
        // `InterestExpense`. The bank lines below `InterestExpenseOperating` count.
        // `InterestExpenseDeposits` counts with each concept below it.
        // `InterestExpenseBorrowings` does not count. Commercial filers report its children, the
        // interest on long-term debt and on short-term borrowings, in the debt note as parts of
        // `InterestExpenseNonoperating`.
        &[
            "InterestExpenseOperating",
            "InterestExpenseBeneficialInterestsIssuedByConsolidatedVariableInterestEntities",
            "InterestExpenseTradingLiabilities",
            "InterestExpenseTrustPreferredSecurities",
        ],
        &["InterestExpenseDeposits"],
        INTEREST_EXPENSE_PART_RULE,
    ),
    PartDefinition::new(
        CashFlowConcept::DepreciationAmortization.element(),
        &["Depreciation"],
        // `AdjustmentForAmortization` counts, and so does each concept below it that amortizes an
        // operating asset, because filers report those in its place. Left out, because filers
        // report them outside D&A as their own add-back:
        // `AccretionAmortizationOfDiscountsAndPremiumsInvestments`,
        // `AmortizationOfDeferredHedgeGains`, `AmortizationOfDeferredLoanOriginationFeesNet`, and
        // `AmortizationOfFinancingCostsAndDiscounts`.
        &[
            "AdjustmentForAmortization",
            "AmortizationAndDepreciationOfDecontaminatingAndDecommissioningAssets",
            "AmortizationOfAcquisitionCosts",
            "AmortizationOfAdvanceRoyalty",
            "AmortizationOfDeferredPropertyTaxes",
            "AmortizationOfDeferredSalesCommissions",
            "AmortizationOfIntangibleAssets",
            "AmortizationOfMortgageServicingRightsMSRs",
            "AmortizationOfNuclearFuelLease",
            "AmortizationOfPowerContractsEmissionCredits",
            "AmortizationOfRateDeferral",
            "AmortizationOfRegulatoryAsset",
            "AmortizationOfValueOfBusinessAcquiredVOBA",
            "CostOfGoodsAndServicesSoldAmortization",
            "DeferredPolicyAcquisitionCostAmortizationExpense",
            "DeferredSalesInducementsAmortizationExpense",
            "FinanceLeaseRightOfUseAssetAmortization",
            "OtherAmortizationOfDeferredCharges",
            "PresentValueOfFutureInsuranceProfitsAmortizationExpense1",
            "OtherDepreciationAndAmortization",
        ],
        &["Depletion"],
        DEPRECIATION_AMORTIZATION_PART_RULE,
    ),
    PartDefinition::new(
        CashFlowConcept::CapitalExpenditures.element(),
        &["PaymentsToAcquirePropertyPlantAndEquipment"],
        &[
            "PaymentForAcquisitionCryptoAsset",
            "PaymentsForDelayedTaxExemptExchange",
            "PaymentsForSoftware",
            "PaymentsToAcquireEquipmentOnLease",
            "PaymentsToAcquireIntangibleAssets",
            "PaymentsToAcquireMineralRights",
            "PaymentsToAcquireOtherProductiveAssets",
        ],
        &[],
        CAPITAL_EXPENDITURES_PART_RULE,
    ),
    PartDefinition::new(
        CashFlowConcept::DividendsPaid.element(),
        &["PaymentsOfDividendsCommonStock"],
        // `PaymentsOfDividendsCommonStock` sits below `PaymentsOfOrdinaryDividends`, next to the
        // dividends of preferred stock and of the noncontrolling interest. The capital
        // distributions sit one level up.
        &[
            "PaymentsOfCapitalDistribution",
            "PaymentsOfDividendsMinorityInterest",
            "PaymentsOfDividendsPreferredStockAndPreferenceStock",
        ],
        &[],
        DIVIDENDS_PAID_PART_RULE,
    ),
    PartDefinition::new(
        CashFlowConcept::ShareRepurchases.element(),
        &["PaymentsForRepurchaseOfCommonStock"],
        &[
            "PaymentsForRepurchaseOfConvertiblePreferredStock",
            "PaymentsForRepurchaseOfInitialPublicOffering",
            "PaymentsForRepurchaseOfOtherEquity",
            "PaymentsForRepurchaseOfPreferredStockAndPreferenceStock",
            "PaymentsForRepurchaseOfPrivatePlacement",
            "PaymentsForRepurchaseOfWarrants",
        ],
        &[],
        SHARE_REPURCHASES_PART_RULE,
    ),
    PartDefinition::new(
        IncomeStatementConcept::OtherComprehensiveIncome.element(),
        &["OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToParent"],
        // The part of the parent is the total minus the share of the noncontrolling interest.
        // The total blocks the part if the filing reports it only with a member.
        &["OtherComprehensiveIncomeLossNetOfTax"],
        &["OtherComprehensiveIncomeLossNetOfTaxPortionAttributableToNoncontrollingInterest"],
        OTHER_COMPREHENSIVE_INCOME_PART_RULE,
    ),
    PartDefinition::new(
        IncomeStatementConcept::ComprehensiveIncome.element(),
        &["ComprehensiveIncomeNetOfTax"],
        // `ComprehensiveIncomeNetOfTax` is the total minus the share of the noncontrolling
        // interest. The total blocks the part if the filing reports it only with a member.
        // `NetIncomeLossAttributableToNoncontrollingInterest` sits below the share, so it blocks.
        &["ComprehensiveIncomeNetOfTaxIncludingPortionAttributableToNoncontrollingInterest"],
        &["ComprehensiveIncomeNetOfTaxAttributableToNoncontrollingInterest"],
        COMPREHENSIVE_INCOME_PART_RULE,
    ),
];
