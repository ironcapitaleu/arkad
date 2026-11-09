---
source: xbrl/src/core/elements.rs
last-verified: 2026-06-14
update-frequency: on-code-change
---

# Canonical Elements

These are the 42 CanonicalElement enum variants the system resolves to. Grouped by financial statement and level.

## Level 1 — SFAC 6 Roots

| Element | Statement | Period Type | SFAC 6 Reference |
| --- | --- | --- | --- |
| Assets | Balance Sheet | Instant | ¶25 |
| Liabilities | Balance Sheet | Instant | ¶35 |
| Equity | Balance Sheet | Instant | ¶49 |
| Revenue | Income Statement | Duration | ¶78 |
| Expenses | Income Statement | Duration | ¶80 |
| Gains | Income Statement | Duration | ¶82 |
| Losses | Income Statement | Duration | ¶83 |
| NetIncome | Income Statement | Duration | Derived |
| OtherComprehensiveIncome | Comprehensive Income | Duration | Part of ¶70 |
| ComprehensiveIncome | Comprehensive Income | Duration | ¶70 |
| OperatingCashFlow | Cash Flow | Duration | Activity |
| InvestingCashFlow | Cash Flow | Duration | Activity |
| FinancingCashFlow | Cash Flow | Duration | Activity |

## Level 2 — Balance Sheet

| Element | Parent | Period Type |
| --- | --- | --- |
| CurrentAssets | Assets | Instant |
| NonCurrentAssets | Assets | Instant |
| CashAndEquivalents | CurrentAssets | Instant |
| ShortTermInvestments | CurrentAssets | Instant |
| AccountsReceivable | CurrentAssets | Instant |
| Inventory | CurrentAssets | Instant |
| PropertyPlantEquipment | NonCurrentAssets | Instant |
| Goodwill | NonCurrentAssets | Instant |
| IntangibleAssets | NonCurrentAssets | Instant |
| CurrentLiabilities | Liabilities | Instant |
| NonCurrentLiabilities | Liabilities | Instant |
| LongTermDebt | NonCurrentLiabilities | Instant |
| RetainedEarnings | Equity | Instant |

## Level 2 — Income Statement

| Element | Parent | Period Type |
| --- | --- | --- |
| CostOfRevenue | Expenses | Duration |
| GrossProfit | Derived | Duration |
| SellingGeneralAdmin | Expenses | Duration |
| ResearchDevelopment | Expenses | Duration |
| InterestExpense | Expenses | Duration |
| OperatingIncome | Derived | Duration |
| EarningsPerShareBasic | NetIncome | Duration |
| EarningsPerShareDiluted | NetIncome | Duration |

## Level 2 — Cash Flow

| Element | Parent | Period Type |
| --- | --- | --- |
| DepreciationAmortization | OperatingCashFlow | Duration |
| CapitalExpenditures | InvestingCashFlow | Duration |
| DividendsPaid | FinancingCashFlow | Duration |
| ShareRepurchases | FinancingCashFlow | Duration |
| ShareBasedCompensation | OperatingCashFlow | Duration |
