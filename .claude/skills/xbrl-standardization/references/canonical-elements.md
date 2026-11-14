---
source: xbrl/src/core/elements/
last-verified: 2026-11-14
update-frequency: on-code-change
---

# Standard Concepts

These are the 36 standard concepts the system resolves to. `CanonicalElementKind` nests them by
statement and disclosure:

- `CanonicalElementKind::Statement(StatementConcept)`: `BalanceSheetConcept`,
  `IncomeStatementConcept`, `CashFlowConcept`.
- `CanonicalElementKind::Disclosure(DisclosureConcept)`: `PerShareConcept`.

A table names a concept as `BalanceSheetConcept::Assets.element()`. Silver stores the variant name,
such as `Assets`, from `Concept::name()`.

`xbrl/DESIGN.md`, "Roll-Up Tree", gives the place of each concept in the tree of its statement.
The top lines rest on the elements of FASB Concepts Statement 8, Chapter 4.

## Balance Sheet (`BalanceSheetConcept`)

Each value is an instant.

| Concept |
| --- |
| Assets |
| CurrentAssets |
| NonCurrentAssets |
| CashAndEquivalents |
| ShortTermInvestments |
| AccountsReceivable |
| Inventory |
| PropertyPlantEquipment |
| Goodwill |
| IntangibleAssets |
| Liabilities |
| Equity |
| CurrentLiabilities |
| NonCurrentLiabilities |
| LongTermDebt |
| RetainedEarnings |

## Income Statement (`IncomeStatementConcept`)

Each value is a duration. The cascade ends with comprehensive income.

| Concept |
| --- |
| Revenue |
| CostOfRevenue |
| GrossProfit |
| SellingGeneralAdmin |
| ResearchDevelopment |
| OperatingIncome |
| InterestExpense |
| NetIncome |
| OtherComprehensiveIncome |
| ComprehensiveIncome |

Note: `Revenue` is narrower than element E80 of Chapter 4. It holds the revenue line the filer
reports. It leaves out the revenues from "other activities" of E84, such as interest income.
`xbrl/DESIGN.md`, "Roll-Up Tree", puts interest income in other non-operating income.

## Cash Flow Statement (`CashFlowConcept`)

Each value is a duration.

| Concept |
| --- |
| OperatingCashFlow |
| InvestingCashFlow |
| FinancingCashFlow |
| DepreciationAmortization |
| ShareBasedCompensation |
| CapitalExpenditures |
| DividendsPaid |
| ShareRepurchases |

## Per Share (`PerShareConcept`)

A disclosure. Its concepts are outside the trees of the statements, and they do not add over
periods.

| Concept |
| --- |
| EarningsPerShareBasic |
| EarningsPerShareDiluted |
