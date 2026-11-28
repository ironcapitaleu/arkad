# ADR 0056: Revenue of Banks, Utilities, and Oil and Gas Producers

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-28 ([STA-623 comment `d4bcbe6b`](https://linear.app/state-machine/issue/STA-623#comment-d4bcbe6b), accessed 2026-11-28) |
| Date | 2026-11-28 |
| Deciders | The project owner |
| Ticket | [STA-623](https://linear.app/state-machine/issue/STA-623), accessed 2026-11-28 |

## Context

The `Revenue` of a filing comes from the alias list `Revenues`, `SalesRevenueNet`, or from a part
rule. The parts are the two `RevenueFromContractWithCustomer…` concepts, `SalesRevenueGoodsNet`,
and `SalesRevenueServicesNet`. Some banks, utilities, and oil and gas producers report their own
revenue lines and no concept of the list or the parts. Their `Revenue` stays open.

The acceptance test A1 (`xbrl/tests/acceptance_roll_ups.rs`) shows 10 open lines on `main` at
`79f7276`. Seven are `Revenue`. The annual values without dimensions of these filings, in million
USD:

| Filing | Revenue lines |
| ------ | ------------- |
| Regions Financial 2013 | `InterestAndDividendIncomeOperating` 3,646, `InterestIncomeExpenseNet` 3,262, `InterestIncomeExpenseAfterProvisionForLoanLoss` 3,124, `NoninterestIncome` 2,019 |
| Regions Financial 2021 | 4,081, 3,914, 4,438, 2,524, in the same order |
| Regions Financial 2025 | 7,073, 4,991, 4,521, 2,535, in the same order. Also `InterestRevenueExpenseNet` 4,991 |
| Evergy 2013 | `ElectricUtilityRevenue` 2,446.3 |
| APA 2013 | `OilAndGasRevenue` 16,054, `OtherSalesRevenueNet` 51, `ExplorationAndProductionRevenue` 16,402 |
| APA 2025 | `RevenuesAndOther` 9,220 and other concepts of the filer |
| KKR 2013 | `InvestmentAdvisoryFees` 763, and fee lines of the filer |

In this document, these terms have a narrow sense:

- **Net interest income** is `InterestIncomeExpenseNet`: interest income less interest expense,
  before the provision for credit losses.
- **Gross revenue of a bank** is interest and dividend income plus noninterest income.
- **Net revenue of a bank** is net interest income plus noninterest income.
- A **line total** is the sum of revenue lines that a filing reports in place of a total.

A bank has no single revenue line. A sum of two lines needs a rule that the alias list and the part
rule do not have. The part rule takes one concept as the whole.

The 2026 release places the bank concepts in two ways:

- The calculation tree `us-gaap-stm-soi4-cal-2026.xml` sums `Revenues` from
  `InterestIncomeExpenseAfterProvisionForLoanLoss` and `NoninterestIncome`.
  `InterestIncomeExpenseNet` sums interest income less `InterestExpenseOperating`.
- The calculation tree `us-gaap-stm-soi1-cal-2026.xml` gives `RevenuesNetOfInterestExpense` as
  `Revenues` less `InterestExpenseOperating`. There, `Revenues` holds the gross interest income.

The filers do not follow the first tree. JPMorgan Chase is the one fixture bank with `Revenues`:

| Filing | `Revenues` | Net interest income + noninterest income | After provision + noninterest income |
| ------ | ---------- | ---------------------------------------- | ------------------------------------ |
| JPMorgan Chase 2013 | 96,606 | 43,319 + 53,287 = 96,606 | 96,381 |
| JPMorgan Chase 2025 | 182,447 | 95,443 + 87,004 = 182,447 | 168,235 |

JPMorgan Chase 2025 also reports `RevenuesNetOfInterestExpense` = 182,447, and its 10-Qs of 2025
report it with no `Revenues`. [ADR 0049](0049-concepts-before-2018-and-the-cash-scope.md),
decision 2, gives `RevenuesNetOfInterestExpense` no mapping and leaves it to this decision.

The utility and producer concepts are not in the 2026 release. In the releases of 2013 and 2017,
`ElectricUtilityRevenue` and `GasDomesticRegulatedRevenue` sum to `UtilityRevenue`.
`UtilityRevenue`, `OilAndGasRevenue`, `OtherSalesRevenueNet`, `SalesRevenueGoodsNet`, and about 30
other lines sum to `SalesRevenueNet`. `ExplorationAndProductionRevenue` is a child of
`OilAndGasRevenue`.

The decision binds the alias list and the part rule of `Revenue` in `us_gaap`, a new rule kind of
the plug-in, the type `ConceptAliases`, and the confidence of a sum of reported lines.

## Decision

1. **The `Revenue` of a bank is its net revenue.** It is net interest income plus noninterest
   income, before the provision for credit losses. JPMorgan Chase, the one fixture bank with
   `Revenues`, reports this measure as `Revenues` in 2013 and 2025.
2. **A line total rule gives the net revenue of a bank.** The rule `us-gaap/total/revenue@1` adds
   `InterestIncomeExpenseNet` and `NoninterestIncome`. It runs only if no alias and no part gives a
   value in the period. Both lines must have a value in the same period, unit, and dimensions.
   `ConceptAliases` gains a field for the lines and the rule ID.
3. **A value of a line total has the confidence `Computed`.** No concept of the filing holds the
   value, so the value is no `Synonym`.
4. **`RevenuesNetOfInterestExpense` joins the alias list of `Revenue`, after `SalesRevenueNet`.**
   A bank reports it as its net revenue. This replaces decision 2 of ADR 0049.
5. **`UtilityRevenue`, `ElectricUtilityRevenue`, and `OilAndGasRevenue` become parts of
   `Revenue`.** The own other parts of each are its siblings, and the siblings of each concept
   between it and `Revenues`, in the trees of 2013 and 2017. The siblings that filers also report
   outside revenue stay out, as for the other parts of `Revenue` today. The new parts follow the
   existing parts, in the order `UtilityRevenue`, `ElectricUtilityRevenue`, `OilAndGasRevenue`.
   `ExplorationAndProductionRevenue` stays out: it is a part of a part.
6. **No value of this decision carries a scope.** Each value measures the revenue that the filers
   report as `Revenues`.
7. **Revenue that a filer reports only with its own concepts stays open.** APA 2025 and KKR 2013
   get no `Revenue`.

## Options Considered

Measured on 2026-11-28 on `main` at `79f7276`. A local copy of `mappings.rs` held decisions 4 and
5, and A1 to A4 ran against it. A scratch test read the bank lines of each fixture 10-K after tiers
1 and 2, and added them where `Revenue` stayed open. Neither was committed.

### The Revenue of a Bank

| Option | Regions 2013 | Regions 2021 | Regions 2025 | Equal to `Revenues` of JPMorgan Chase 2013 and 2025 |
| ------ | ------------ | ------------ | ------------ | --------------------------------------------------- |
| **A. Net (chosen)** | 5,281 | 6,438 | 7,526 | Yes, both |
| B. Gross | 5,665 | 6,605 | 9,608 | No. JPMorgan Chase reports a net `Revenues` |
| C. After provision, as the tree of `soi4` | 5,143 | 6,962 | 7,056 | No: 96,381 and 168,235 |
| D. Open | open | open | open | Not applicable |

Values in million USD. Each of A to C fills the 3 lines of Regions Financial in A1.

- A gives one meaning to `Revenue` across banks: the `Revenues` of a bank that reports a total
  equals the line total of a bank that does not.
- B gives a bank with a total and a bank without one two different measures. Its value rises with
  interest rates, as Regions 2025 shows: 9,608 against 7,526.
- C follows the 2026 tree, but no fixture bank reports a `Revenues` after provision. The provision
  is an expense, and it moves `Revenue` with the credit cycle: Regions 2021 released reserves.
- D leaves 3 of the 7 open lines open.

### The Rule for the Bank Lines

| Option | Values that change meaning | New standard concepts | A check for a bank with `Revenues` |
| ------ | -------------------------- | --------------------- | ---------------------------------- |
| **A. Line total of the plug-in (chosen)** | None | None | No |
| B. A closed group `Revenue` = net interest income + noninterest income | Net interest income of commercial filers | Two | Yes |
| C. A sum along the calculation tree (Tier 4) | None | None | No |

- B needs `InterestIncomeExpenseNet` as a standard concept. Commercial filers report it as their
  interest outside operations: AMD 2025 215, Walmart 2013 −2,216, Medtronic 2025 511, and Baker
  Hughes 2025 −222 million USD. The concept would mean two things.
- C sums the children of `Revenues` in one tree. The tree of `soi4` gives the value after
  provision, option C of the table above.
- A part rule takes one concept as the whole, so it cannot add two lines.

### The Confidence of a Line Total

| Option | What the confidence tells a reader |
| ------ | ---------------------------------- |
| **A. `Computed` (chosen)** | The filing reports no concept that holds the value |
| B. `Synonym` | One reported concept holds the value, which is false |

### The Other Concepts

| Change | A1 lines filled | Other tests that change |
| ------ | --------------- | ----------------------- |
| `RevenuesNetOfInterestExpense` as an alias | 0 | A3: JPMorgan Chase 2025 gets a fourth quarter of `Revenue`, 45,798 million USD, from the 10-K `Revenues` less the nine-month `RevenuesNetOfInterestExpense` |
| `UtilityRevenue` and `ElectricUtilityRevenue` as parts | 1: Evergy 2013, 2,446.3 million USD, `Synonym` | None |
| `OilAndGasRevenue` as a part | 0: `OtherSalesRevenueNet` blocks it for APA 2013 | None |

- AES 2013 and Ameren 2013 report `ElectricUtilityRevenue` next to `Revenues`. The alias wins, so
  their lines do not change.
- APA 2013 reports `ExplorationAndProductionRevenue` = 16,402 = 16,054 + 51 + 297. The 297 has the
  concept `GasGatheringTransportationMarketingAndProcessingCosts`, a cost. So no reported concept
  is the whole by the trees.

## Consequences

- A1 fills 4 of its 7 open `Revenue` lines: Regions Financial 2013, 2021, and 2025 by the line
  total, and Evergy 2013 by the part rule. A2 lists no open `Revenue` for these filings.
- APA 2013, APA 2025, and KKR 2013 keep `Revenue` open. APA 2025 and KKR 2013 report revenue with
  concepts of the filer. KKR 2013 reports `InvestmentAdvisoryFees`, a fee line, which no tree
  places below a revenue total.
- `NoninterestIncome` stays an other part of `Revenue`. Regions 2021 reports a contract revenue
  part that is a part of noninterest income only. The other part blocks it, and the line total
  gives the value.
- `ConceptAliases` gains one field, and `pick` in `core::alias` tries the line total after the
  parts. The `Taxonomy` trait does not change.
- `Computed` values come from tiers 1 and 2 for the first time. The tier table of `xbrl/DESIGN.md`
  names the line total next to the sum along the tree.
- A ratio on `Revenue` of a bank, such as a margin, uses net revenue. A reader who wants gross
  revenue reads `InterestAndDividendIncomeOperating` from the as-is layer.
- The line total needs both lines. Only the bank filings report `NoninterestIncome`: 8 of the 105
  Notes fixture filings, all of JPMorgan Chase and Regions Financial.
- The line total runs with the parts, on each set of dimensions. An alias or a part blocks it only
  in its own set of dimensions. A bank that reports both lines for a segment gets a `Computed`
  segment value of `Revenue`.
- Not covered: a bank that reports a contract revenue part and `InterestIncomeExpenseNet`, but no
  `NoninterestIncome`. The part gives only the contract revenue as `Revenue`. Commercial filers
  report `InterestIncomeExpenseNet` outside revenue, so it cannot block the part.
- Not covered: a non-bank filer, such as a broker or an insurer, with no alias and no part, that
  reports `InterestIncomeExpenseNet` outside operations and `NoninterestIncome`. Its `Revenue` is
  the sum of the two, with no finding.
- Not covered: a filer that reports `Revenues` as gross revenue next to
  `RevenuesNetOfInterestExpense`. `Revenues` comes first in the list, so the value is gross. If its
  10-Qs report only `RevenuesNetOfInterestExpense`, the fourth quarter subtracts a net nine months
  from a gross year.
- Not covered: a filer that reports `SalesRevenueGoodsNet` or `SalesRevenueServicesNet` next to a
  new part. The goods or services line comes first in the list, so `Revenue` is that line alone.
- Not covered: a utility that reports `ElectricUtilityRevenue` and a gas line, but no
  `UtilityRevenue`. The gas line blocks the part, and `Revenue` stays open.

## Open Questions

- The table of removed US GAAP elements will list the siblings of `UtilityRevenue` and
  `OilAndGasRevenue` in every release before 2018. A sibling that the trees of 2013 and 2017 miss
  joins the own other parts.
- The siblings of `SalesRevenueGoodsNet` and `SalesRevenueServicesNet` stay open, as ADR 0049
  says. Decision 5 does not add the new parts to their own other parts.
- A check of `Revenues` against the line total of a bank, as JPMorgan Chase allows, waits for a
  rule that compares two values of one standard concept.
- A filer type that reports revenue in lines with no total, such as an insurer or an asset
  manager, needs an amendment of decision 2 for a new list of lines.

## Sources

- `xbrl/src/tests/fixtures/data/notes/<accession>/num.tsv.gz` at `79f7276`: the annual values
  without dimensions and without a co-registrant of the fixture 10-Ks. Regions Financial
  `0001281761-14-000009`, `0001281761-22-000016`, `0001281761-26-000019`. Evergy
  `0001143068-14-000021`. APA `0001193125-14-077668`, `0001841666-26-000015`. KKR
  `0001047469-14-001189`. JPMorgan Chase `0000019617-14-000289`, `0001628280-26-008131`.
- `xbrl/taxonomy/us-gaap/2026/stm/` at `79f7276`: the files `us-gaap-stm-soi4-cal-2026.xml` and
  `us-gaap-stm-soi1-cal-2026.xml`, and the schema `elts/us-gaap-2026.xsd` for the removed concepts.
- The FASB releases of 2013 and 2017 ([us-gaap-2013-01-31.zip](https://xbrl.fasb.org/us-gaap/2013/us-gaap-2013-01-31.zip)
  and [us-gaap-2017-01-31.zip](https://xbrl.fasb.org/us-gaap/2017/us-gaap-2017-01-31.zip), accessed
  2026-11-28): the file `stm/us-gaap-stm-soi-cal-*.xml` of each.
- `xbrl/tests/acceptance_roll_ups.rs`, `xbrl/tests/acceptance_open_roll_ups.rs`, and
  `xbrl/tests/acceptance_fourth_quarter.rs` at `79f7276`, run on 2026-11-28 with the local copy of
  `mappings.rs`.
- [ADR 0049](0049-concepts-before-2018-and-the-cash-scope.md), decision 2.
