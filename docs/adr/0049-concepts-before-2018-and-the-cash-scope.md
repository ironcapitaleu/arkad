# ADR 0049: Concepts Before 2018 and the Scope of Cash

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project owner confirms |
| Date | 2026-11-27 |
| Deciders | The project owner |
| Ticket | [STA-541](https://linear.app/state-machine/issue/STA-541), accessed 2026-11-27 |

## Context

The backfill reads 10-Ks from fiscal year 2009. Two standards changed the concepts that filers use
from 2018:

- ASC 606 replaced the revenue concepts. The concepts `SalesRevenueNet`, `SalesRevenueGoodsNet`,
  and `SalesRevenueServicesNet` are not in the 2026 release.
- ASU 2016-18 moved restricted cash into the cash of the cash flow statement. Before it, the
  statement reconciled cash and cash equivalents only. The concepts
  `CashAndCashEquivalentsPeriodIncreaseDecrease`, `CashPeriodIncreaseDecrease`, and
  `EffectOfExchangeRateOnCashAndCashEquivalents` measure that narrower cash. They are not in the
  2026 release.

The alias lists hold `SalesRevenueNet` only. The MVP Decision Spike resolved 90 to 92% of S&P 500
10-Ks before 2018 with the older concepts, and 72 to 80% without them.

In this document, **scope** means which balance a cash value measures. A value of
`NetChangeInCash` with the full scope is the change in cash, cash equivalents, and restricted cash.
A value with the narrow scope leaves out restricted cash.

The acceptance test A1 (`xbrl/tests/acceptance_roll_ups.rs`) lists the 11 roll-ups of the spike
for each fixture 10-K. On `main` at `e398432`, 68 lines are `open`. The fixture set holds 34
10-Ks before 2018: one per company for fiscal year 2013 (Amazon 2014), and Supermicro 2017.

The decision binds the alias lists and part rules of `us_gaap`, the `StandardizedFact` type, the
silver table `standardized_facts`, the closed group `net-change-in-cash`, and the design of the
check of `NetChangeInCash` against two balances.

## Decision

1. **`SalesRevenueGoodsNet` and `SalesRevenueServicesNet` become parts of `Revenue`.** Each is an
   other part of the other, so a filing that reports both gets no value from them. The alias
   `SalesRevenueNet` covers that filing. The values carry no scope: both concepts measure revenue.
2. **`RevenuesNetOfInterestExpense` gets no mapping.** In the 2026 calculation tree it is
   `Revenues` minus `InterestExpenseOperating`, a different amount. The industry revenue design
   decides it.
3. **The three `…ContinuingOperations` totals become parts of the cash flow totals.** The
   `CashProvidedByUsedIn…ActivitiesDiscontinuedOperations` flows are their other parts, as the
   2026 calculation tree places them. `NetCashProvidedByUsedInContinuingOperations` gets no
   mapping.
4. **The concepts of the narrow cash map with a scope on the value.**
   `CashAndCashEquivalentsPeriodIncreaseDecrease` and `CashPeriodIncreaseDecrease` join the alias
   list of `NetChangeInCash`. `EffectOfExchangeRateOnCashAndCashEquivalents` joins the alias list
   of `ExchangeRateEffect`. `EffectOfExchangeRateOnCashAndCashEquivalentsContinuingOperations`
   becomes a part of `ExchangeRateEffect`, with the other part
   `EffectOfExchangeRateOnCashAndCashEquivalentsDiscontinuedOperations`. Each value from these concepts carries the scope
   `ExcludesRestrictedCash`.
5. **The scope is a field of `StandardizedFact`.** `scope: Option<Scope>`, where `Scope` is a
   `#[non_exhaustive]` enum with one variant, `ExcludesRestrictedCash`. `None` means the full scope
   of the standard concept. Silver `standardized_facts` gets a nullable `scope` column.
6. **A closed group derives and checks only values of one scope.** Only `NetChangeInCash` and
   `ExchangeRateEffect` carry a scope. The three cash flow totals carry none, and they count with
   either scope (see "Consequences"). A value derived from `net-change-in-cash` takes the scope of
   the members that carry one. If `NetChangeInCash` and `ExchangeRateEffect` differ in scope, the
   group gets no check, derives nothing, and reports a `MixedScopes` finding.
7. **A new standard concept, `CashAndRestrictedCash`, holds cash at the end of the period.** It is
   a top line of the cash flow statement with instant values. Its alias is
   `CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsIncludingDisposalGroupAndDiscontinuedOperations`.
   Its part is `CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents`, with the other
   part `CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsDisposalGroupIncludingDiscontinuedOperations`.
   A period before ASU 2016-18 gets no value. The balance sheet line stays in
   `CashAndEquivalents`.
8. **Banks get no new rule for `CashAndEquivalents`.** `CashAndDueFromBanks` and `DueFromBanks` get
   no mapping. After ASU 2016-18, `CashAndRestrictedCash` holds the cash of a bank.
9. **No rule derives `NetChangeInCash` from two balances.** The difference of two balances is the
   input of a check only. The check belongs to a separate design.

## Options Considered

Measured on 2026-11-27 on `main` at `e398432`. The fixture rows come from
`xbrl/src/tests/fixtures/data/`. A local copy of `mappings.rs` held the aliases and parts of
decisions 1, 3, and 4 as plain entries, and A1 ran against it. The copy was not committed.

### The Narrow Cash Concepts

| Option | A1 lines filled | 2018 break | Closed group before 2018 |
| ------ | --------------- | ---------- | ------------------------ |
| **A. Scope on the value (chosen)** | 33 (31 by alias, 2 derived) | Visible on each value | Checks and derives within one scope |
| B. `Synonym` | 33 | Not visible | Mixes scopes with no sign |
| C. No mapping | 0 | No values before 2018 | Derives nothing before 2018: the exchange rate effect has no value |
| D. A new standard concept for the narrow scope | 33 | Visible as a second series | `net-change-in-cash` needs two parents |

- A keeps the series whole and gives each value one meaning. A reader filters on the scope.
- B breaks the series in 2018 with no sign. `NetChangeInCash` of 2017 and of 2018 measure different
  balances.
- C leaves `NetChangeInCash` open for 31 of the 34 10-Ks before 2018. All 31 report
  `CashAndCashEquivalentsPeriodIncreaseDecrease`.
- D splits one line of the statement into two concepts, and the group changes its parent by era.

### Where the Scope Lives

| Option | Silver schema | Reader | Check of a group |
| ------ | ------------- | ------ | ---------------- |
| **A. A field and a column (chosen)** | One nullable column | Filters on `scope` | Compares one field |
| B. A separate rule ID per scope | No change | Must know which rule IDs carry the narrow scope | Needs a table from rule ID to scope |

### Cash at the End of the Period

| Option | Values after 2018 | Values before 2018 | Copies of another concept |
| ------ | ----------------- | ------------------ | ------------------------- |
| **A. New concept, ASU 2016-18 concepts only (chosen)** | 36 of 36 fixture 10-Ks of fiscal years 2018 and later | None | None |
| B. New concept, plus the balance sheet line with the scope before 2018 | 36 of 36 | Most | The value of `CashAndEquivalents` before 2018 |
| C. No concept | None | None | None |

- B copies `CashAndEquivalents` before 2018, and for a bank such as JPMorgan Chase 2013, the
  statement reconciles `CashAndDueFromBanks`, a third balance.
- C leaves the check of the change in cash with no balance with restricted cash.

### Bank Cash

In the 2026 tree, `CashAndCashEquivalentsAtCarryingValue` is `CashAndDueFromBanks` plus
`InterestBearingDepositsInBanks` plus `CashEquivalentsAtCarryingValue`. `CashAndDueFromBanks` is
`Cash` plus `DueFromBanks`.

| Option | Fixture 10-Ks that gain a value |
| ------ | ------------------------------- |
| **A. No new rule (chosen)** | 0 |
| B. `CashAndDueFromBanks` as a part of `CashAndEquivalents` | 0. Each of the 5 bank 10-Ks reports `InterestBearingDepositsInBanks` next to it, which blocks the part |
| C. `DueFromBanks` as a part | 0. It is a part of a part, and `Cash` or the deposits block it |

- The sum of the two bank lines fills the bank case, for example Regions Financial 2013:
  1,661 + 3,612 = 5,273 million USD, its `CashAndCashEquivalentsAtCarryingValue`. A sum along the
  calculation tree is no part rule.

### A Change in Cash From Two Balances

| Option | Gain | Cost |
| ------ | ---- | ---- |
| **A. Check only (chosen)** | A finding when the reported change and the balances disagree | No value for a filing with no change line |
| B. Derivation rule | A value for a filing with no change line | The check then compares the value with itself |

- Before 2018 the balance sheet line can differ from the cash of the statement: banks, and cash of
  discontinued operations. A derived value from the wrong balance has no check left to catch it.

## Consequences

- A1 changes 58 of its 68 open lines (measured with the local copy):
  - Decision 1: Coca-Cola 2013, Deckers 2013, and General Mills 2013 get `Revenue`.
  - Decision 3: 22 cash flow totals of 3M, AbbVie, Air Products, Deckers, Digital Realty,
    ExxonMobil, and KKR 2013, and of Air Products 2025.
  - Decision 4: `NetChangeInCash` for 31 10-Ks by alias, and for AbbVie 2013 and Deckers 2013 by
    `core/roll-up/net-change-in-cash@1`.
- Ten lines stay open: Air Products 2013 operating and investing cash flow (discontinued
  operations block the part), KKR 2013 revenue and change in cash, APA 2013 and 2025 revenue,
  Evergy 2013 revenue, and Regions Financial 2013, 2021, and 2025 revenue.
- The acceptance tests A3 and A4 still pass with the local copy.
- The three cash flow totals keep one meaning across 2018: the total of each activity. ASU 2016-18
  moved restricted cash transfers out of them, as ASU 2016-15 moved other lines in the same year.
  A value carries no scope for a change of the lines inside a total.
- `ExchangeRateEffect` before 2018 now has values, so `net-change-in-cash` derives in a filing
  before 2018 only within the narrow scope.
- A 10-K of 2018 restates the 2017 comparative with the full scope. Dedup takes the newest filing,
  so a period can mix values of two scopes. `MixedScopes` reports it.
- Silver gains one column and one finding kind. A row stored before the change has a null `scope`,
  which reads as the full scope. A rebuild from the as-is layer sets it.
- Not covered: `CashAndCashEquivalentsPeriodIncreaseDecreaseExcludingExchangeRateEffect`. No
  fixture 10-K reports it.

## Open Questions

- The 2026 tables cannot place the concepts of decisions 1 and 4, because the 2026 release removed
  them. The fixture rows support the placement: Amazon 2014, Baker Hughes 2013, and Intuitive
  Surgical 2013 report goods plus services equal to `SalesRevenueNet`. 3M 2013 reports
  5,817 − 856 − 5,246 − 17 = −302 million USD as
  `CashAndCashEquivalentsPeriodIncreaseDecrease`. The table of removed elements confirms the
  placement in the earlier releases. A difference there reopens decisions 1 and 4.
- The other parts of `SalesRevenueGoodsNet` and `SalesRevenueServicesNet` are only each other. A
  filer that splits revenue into goods and a third line, such as `OtherSalesRevenueNet`, gets the
  goods line as `Revenue` with no finding. The table of removed elements lists the siblings of the
  two concepts. A sibling there joins the other parts.
- No fixture 10-K reports `EffectOfExchangeRateOnCashAndCashEquivalentsDiscontinuedOperations`. The
  table of removed elements confirms its name.
- The sum of `CashAndDueFromBanks` and `InterestBearingDepositsInBanks` waits for a rule along the
  calculation tree.
- A second scope, such as cash of discontinued operations, adds a variant to `Scope` and reopens
  decision 6.

## Sources

- `xbrl/src/us_gaap/taxonomy.rs` at `e398432`: the 2026 calculation arcs, and the absence of the
  removed concepts.
- `xbrl/src/tests/fixtures/data/` at `e398432`: the Notes rows of the fixture 10-Ks.
- `xbrl/tests/acceptance_roll_ups.rs` at `e398432`: the 68 open lines of A1.
- [Linear document "Carry-Over of the MVP Decision Spike: Findings"](https://linear.app/state-machine/document/carry-over-of-the-mvp-decision-spike-findings-38a199f5009c),
  accessed 2026-11-27, "Inventory" items 13, 15, 18, and 19, and "Alternatives": the spike shares
  before 2018 and the three options.
- [STA-541 comment `d2a35b0a`](https://linear.app/state-machine/issue/STA-541#comment-d2a35b0a),
  accessed 2026-11-27: the split of the work with the design of the check against two balances and
  the ticket that adds the concept of decision 7.
- The owner chose the options of decisions 4, 5, and 7 in the session of 2026-11-27.
