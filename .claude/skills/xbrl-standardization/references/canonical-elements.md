---
source: xbrl/src/core/elements/
last-verified: 2026-11-16
update-frequency: on-code-change
---

# Standard Concepts

These are the 59 standard concepts the system resolves to. `CanonicalElementKind` nests them by
statement and disclosure:

- `CanonicalElementKind::Statement(StatementConcept)`: `BalanceSheetConcept`,
  `IncomeStatementConcept`, `CashFlowConcept`.
- `CanonicalElementKind::Disclosure(DisclosureConcept)`: `PerShareConcept`, `DebtMaturityConcept`.

A table names a concept as `BalanceSheetConcept::Assets.element()`. Silver stores the variant name,
such as `Assets`, from `Concept::name()`.

Each concept of a statement has a placement (`Placed::placement()`): `Top`, or a child of one group
with a weight of +1 or −1. A child marked "zero if absent" counts as zero in its closed group when it
has no value ([ADR 0019](../../../../docs/adr/0019-zero-if-absent-in-the-placement.md)). `xbrl/DESIGN.md`, "Roll-Up Tree", gives the tables. The top lines rest on
the elements of FASB Concepts Statement 8, Chapter 4.

## Balance Sheet (`BalanceSheetConcept`)

Each value is an instant.

| Concept | Placement |
| --- | --- |
| Assets | Top |
| CurrentAssets | `assets`, +1 |
| NonCurrentAssets | `assets`, +1 |
| CashAndEquivalents | `current-assets`, +1 |
| ShortTermInvestments | `current-assets`, +1 |
| AccountsReceivable | `current-assets`, +1 |
| Inventory | `current-assets`, +1 |
| PropertyPlantEquipment | `non-current-assets`, +1 |
| Goodwill | `non-current-assets`, +1 |
| IntangibleAssets | `non-current-assets`, +1 |
| LiabilitiesAndEquity | Top |
| Liabilities | `liabilities-and-equity`, +1 |
| TemporaryEquity | `liabilities-and-equity`, +1, zero if absent |
| Equity | `liabilities-and-equity`, +1 |
| CurrentLiabilities | `liabilities`, +1 |
| NonCurrentLiabilities | `liabilities`, +1 |
| AccountsPayable | `current-liabilities`, +1 |
| ShortTermDebt | `current-liabilities`, +1 |
| LongTermDebt | `non-current-liabilities`, +1 |
| EquityAttributableToParent | `equity`, +1 |
| NoncontrollingInterest | `equity`, +1, zero if absent |
| AdditionalPaidInCapital | `equity-attributable-to-parent`, +1 |
| RetainedEarnings | `equity-attributable-to-parent`, +1 |
| TreasuryStock | `equity-attributable-to-parent`, −1 |
| AccumulatedOtherComprehensiveIncome | `equity-attributable-to-parent`, +1 |

## Income Statement (`IncomeStatementConcept`)

Each value is a duration. The cascade ends with comprehensive income.

| Concept | Placement |
| --- | --- |
| Revenue | Top |
| CostOfRevenue | `gross-profit`, −1 |
| GrossProfit | Top |
| OperatingExpenses | `operating-income`, −1 |
| SellingGeneralAdmin | `operating-expenses`, +1 |
| ResearchDevelopment | `operating-expenses`, +1 |
| OperatingIncome | Top |
| InterestExpense | `pretax-income`, −1 |
| PretaxIncome | Top |
| IncomeTaxExpense | `net-income`, −1 |
| NetIncome | Top |
| NetIncomeAttributableToParent | `net-income-attribution`, +1 |
| NetIncomeAttributableToNoncontrollingInterest | `net-income-attribution`, +1, zero if absent |
| OtherComprehensiveIncome | `comprehensive-income`, +1 |
| ComprehensiveIncome | Top |

Note: `Revenue` is narrower than element E80 of Chapter 4. It holds the revenue line the filer
reports. It leaves out the revenues from "other activities" of E84, such as interest income.
`xbrl/DESIGN.md`, "Roll-Up Tree", puts interest income in other non-operating income.

## Cash Flow Statement (`CashFlowConcept`)

Each value is a duration.

| Concept | Placement |
| --- | --- |
| NetChangeInCash | Top |
| OperatingCashFlow | `net-change-in-cash`, +1 |
| InvestingCashFlow | `net-change-in-cash`, +1 |
| FinancingCashFlow | `net-change-in-cash`, +1 |
| ExchangeRateEffect | `net-change-in-cash`, +1, zero if absent |
| DepreciationAmortization | `operating-cash-flow`, +1 |
| ShareBasedCompensation | `operating-cash-flow`, +1 |
| CapitalExpenditures | `investing-cash-flow`, −1 |
| DividendsPaid | `financing-cash-flow`, −1 |
| ShareRepurchases | `financing-cash-flow`, −1 |

## Per Share (`PerShareConcept`)

A disclosure. Its concepts have no placement, and they do not add over periods.

| Concept |
| --- |
| EarningsPerShareBasic |
| EarningsPerShareDiluted |

## Debt Maturities (`DebtMaturityConcept`)

A disclosure. Its concepts have no placement. Each value is an instant. The six buckets add up to
the total within ±5% (`Summed`), and a bucket has values only at fiscal year ends (`Timed`).

| Concept | Role | Cadence |
| --- | --- | --- |
| LongTermDebtIncludingCurrentMaturities | Total | EveryPeriod |
| DebtDueInYearOne | Part | FiscalYearEnd |
| DebtDueInYearTwo | Part | FiscalYearEnd |
| DebtDueInYearThree | Part | FiscalYearEnd |
| DebtDueInYearFour | Part | FiscalYearEnd |
| DebtDueInYearFive | Part | FiscalYearEnd |
| DebtDueAfterYearFive | Part | FiscalYearEnd |

## Groups

`statement::groups()` lists a group only if its parent, its start if it is a step, and at least
one child are standard concepts. Each statement lists its groups in the order of the first row
that names each group in its table
([ADR 0013](../../../../docs/adr/0013-order-of-the-groups.md)):
`assets`, `current-assets`, `non-current-assets`, `liabilities-and-equity`, `liabilities`,
`current-liabilities`, `non-current-liabilities`, `equity`, `equity-attributable-to-parent`,
`gross-profit`, `operating-income`, `operating-expenses`, `pretax-income`, `net-income`,
`net-income-attribution`, `comprehensive-income`, `net-change-in-cash`, `operating-cash-flow`,
`investing-cash-flow`, `financing-cash-flow`.
