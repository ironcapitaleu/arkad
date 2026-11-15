# ADR 0020: A Start With No Value Counts as the Step That Makes It

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([STA-524 comment `8b6a7953`](https://linear.app/state-machine/issue/STA-524/feature-the-generic-statement-and-the-typed-views-of-the-statements#comment-8b6a7953)) |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-524](https://linear.app/state-machine/issue/STA-524/feature-the-generic-statement-and-the-typed-views-of-the-statements) |

## Context

`Statement::build` in `xbrl/src/core/statement/mod.rs` computes the other line of each open group:
the parent less the start and the weighted children. If a member has no value and is the parent of
a group, the member is **transparent**: its own members count in its place.

A concept can be the parent of two groups. Today one concept is: `NetIncome` is the parent of the
step `net-income` (from `PretaxIncome`, less `IncomeTaxExpense`) and of the closed breakdown
`net-income-attribution` (`NetIncomeAttributableToParent` plus
`NetIncomeAttributableToNoncontrollingInterest`). `NetIncome` is also the start of the step
`comprehensive-income`.

So if `NetIncome` has no value and `ComprehensiveIncome` has one, the other line of
`comprehensive-income` must replace `NetIncome` by one of its two groups. Both groups equal
`NetIncome` if every member has a value. They differ when some members have no value.

The `NetIncome` part rule gives no value if the filer reports the noncontrolling share
(`NetIncomeLossAttributableToNoncontrollingInterest`) and no `ProfitLoss`. Then
`NetIncomeAttributableToParent` can still have a value through its alias `NetIncomeLoss`.

The open PR ironcapitaleu/arkad#568 (STA-523) derives a value from a closed group with exactly one
unknown member. It derives `NetIncome` from `net-income-attribution` if both of its members are
known. The noncontrolling share counts as known if it is absent. The step `net-income` is open, so
it derives nothing.

## Decision

1. **A transparent member counts as the step that makes it, and as its first group in table order
   only if it has no step.** For `NetIncome`, this is `net-income`, so `comprehensive-income`
   starts from `PretaxIncome` less `IncomeTaxExpense`. The step goes back further if those have no
   value too.
2. **A member that is the parent of no group of the statement counts as zero.** An example is
   `NetIncome` in the cash flow statement, so the other line of `operating-cash-flow` holds it.

## Options Considered

Measured on 2026-11-15 with the SEC frames API, frame `CY2025Q3`, unit USD, one fact per filer
with no dimensions. "Has no `NetIncome`" means no `ProfitLoss`, and no `NetIncomeLoss` or a
noncontrolling share next to it. "Step" means `IncomeLossFromContinuingOperationsBeforeIncomeTaxesExtraordinaryItemsNoncontrollingInterest`
and `IncomeTaxExpenseBenefit`. "Split" means `NetIncomeLoss` and the noncontrolling share.

| Filers | Count |
| ------ | ----- |
| Report comprehensive income (`ComprehensiveIncomeNetOfTax` or the total with the noncontrolling interest) | 3,511 |
| … and have no `NetIncome` | 23 |
| … step only | 9 |
| … split only | 3 |
| … both | 9 |
| … neither | 2 |

| Option | Filers it serves (of 23) | After STA-523 |
| ------ | ------------------------ | ------------- |
| **A: the step (chosen)** | 18 | the 12 with the split get a derived `NetIncome`, so they need no group. A serves the 9 left with the step only |
| B: the split | 12 | the same 12 get a derived `NetIncome`. B serves none of the rest |
| C: the group whose members have values | 21 | the same as A, with more code and tests |

- A won: STA-523 fills `NetIncome` for each filer that the split could serve. The cases left need
  the step. A also follows the cascade of the income statement, where each step starts from the
  previous subtotal.
- B serves fewer filers before STA-523 and none after it.
- C serves 3 more filers before STA-523, and none more after it, at the cost of a second rule.

## Consequences

- The fixture filing `0001628280-26-068246` never reaches this case: it reports no comprehensive
  income. The unit test `should_start_the_step_from_the_previous_subtotal_when_the_start_has_no_value`
  covers the rule on `GrossProfit`, which has only a step.
- Before STA-523 merges, a filer with the split and no pretax income gets an other line of
  `comprehensive-income` that goes back to an earlier subtotal. The other line still adds up.
- A new concept that is the parent of a step and a breakdown follows the same rule.

## Open Questions

- The count is one quarter and one alias of pretax income. A measurement over more frames, or with
  every alias, can change the counts. If the split then serves many filers that STA-523 does not
  fill, this decision reopens.

## Sources

- `xbrl/src/core/statement/mod.rs`, `Tree::source_of` and `Layout::add`, on branch
  `feat/statement-views` at `350e480`.
- `xbrl/src/us_gaap/mappings.rs`: the part rule of `NetIncome` on `main` at `97dd816`.
- `xbrl/DESIGN.md`, "Derivation From a Closed Group", on branch `feat/roll-up-derivation`
  (ironcapitaleu/arkad#568), 2026-11-15.
- SEC frames API, `https://data.sec.gov/api/xbrl/frames/us-gaap/<concept>/USD/CY2025Q3.json`, for
  `NetIncomeLoss`, `ProfitLoss`, `NetIncomeLossAttributableToNoncontrollingInterest`,
  `ComprehensiveIncomeNetOfTax`,
  `ComprehensiveIncomeNetOfTaxIncludingPortionAttributableToNoncontrollingInterest`,
  `IncomeLossFromContinuingOperationsBeforeIncomeTaxesExtraordinaryItemsNoncontrollingInterest`,
  and `IncomeTaxExpenseBenefit`, fetched 2026-11-15.
