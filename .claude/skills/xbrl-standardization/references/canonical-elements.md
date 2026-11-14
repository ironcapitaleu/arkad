---
source: xbrl/src/core/elements.rs
last-verified: 2026-11-14
update-frequency: on-code-change
---

# Canonical Elements

These are the 42 CanonicalElement enum variants the system resolves to. Grouped by financial statement and level.

## Level 1 — Roots

The roots are the elements of FASB Concepts Statement 8, Chapter 4. The paragraph numbers below
are those of Chapter 4.

| Element | Statement | Period Type | Chapter 4 Paragraph |
| --- | --- | --- | --- |
| Assets | Balance Sheet | Instant | E16 |
| Liabilities | Balance Sheet | Instant | E37 |
| Equity | Balance Sheet | Instant | E61 |
| Revenue | Income Statement | Duration | E80, see the note below |
| Expenses | Income Statement | Duration | E81 |
| Gains | Income Statement | Duration | E82 |
| Losses | Income Statement | Duration | E83 |
| NetIncome | Income Statement | Duration | Derived |
| OtherComprehensiveIncome | Comprehensive Income | Duration | Part of E75 |
| ComprehensiveIncome | Comprehensive Income | Duration | E75 |
| OperatingCashFlow | Cash Flow | Duration | Activity |
| InvestingCashFlow | Cash Flow | Duration | Activity |
| FinancingCashFlow | Cash Flow | Duration | Activity |

Note: the `Revenue` root is narrower than E80. It holds the revenue line the filer reports. It
leaves out the revenues from "other activities" of E84, such as interest income. `xbrl/DESIGN.md`,
"Roll-Up Tree", puts interest income in other non-operating income.

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
| EarningsPerShareBasic | NetIncome (the part of the parent, `NetIncomeLoss`) | Duration |
| EarningsPerShareDiluted | NetIncome (the part of the parent, `NetIncomeLoss`) | Duration |

## Level 2 — Cash Flow

| Element | Parent | Period Type |
| --- | --- | --- |
| DepreciationAmortization | OperatingCashFlow | Duration |
| CapitalExpenditures | InvestingCashFlow | Duration |
| DividendsPaid | FinancingCashFlow | Duration |
| ShareRepurchases | FinancingCashFlow | Duration |
| ShareBasedCompensation | OperatingCashFlow | Duration |
