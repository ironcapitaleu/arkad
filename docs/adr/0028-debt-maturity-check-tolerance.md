# ADR 0028: The Debt Maturity Check Uses a Tolerance of 5%

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-503](https://linear.app/state-machine/issue/STA-503) |

## Context

`core/check/debt-maturities@1` compares the sum of the six buckets of the maturity ladder with
`LongTermDebtIncludingCurrentMaturities`, the alias of `us-gaap:LongTermDebt`. The 2026
calculation tree makes `LongTermDebt` the parent of the six buckets.

A check of a filer's own sums uses ±0.1% of the largest absolute value in the group
(`xbrl/DESIGN.md`, "5) Validation"). That tolerance covers rounding. The ladder differs from the
total by more than rounding. The buckets hold principal. The total holds the carrying amount, net
of discounts, premiums, issuance costs, and hedge adjustments.

The tolerance decides how many filers get a `MaturityMismatch` finding. It rests on a measurement
that changes with new filings.

## Decision

1. **The check uses ±5% of the largest absolute value in the group.** A difference above it is a
   `MaturityMismatch` finding.
2. **The check runs only if all six buckets and the total have a value.** A missing bucket never
   counts as zero.
3. **The check derives no value.** The ladder has no other line.

## Options Considered

Measured on 2026-11-15 in the SEC frames API, frame `CY2024Q4I`, facts without dimensions. The
bucket is the fiscal-year concept, or the rolling concept if the filer tags only that.

| Option | Filers checked | Findings |
| ------ | -------------- | -------- |
| **A (chosen): ±5%, all six buckets** | 757 | 71 (9%) |
| B: ±0.1%, all six buckets | 757 | 464 (61%) |
| C: ±1%, all six buckets | 757 | 240 (32%) |
| D: ±5%, a missing bucket as zero | 1,387 | 285 (21%) |
| E: no check | 0 | 0 |

- A flags a ladder that misses a part of the debt, or a bucket with the wrong scale. Boeing is an
  example: its ladder sums to 17,859 million against 53,625 million of `LongTermDebt`.
- B flags most filers. A finding on most filers carries no signal.
- Adding the net discount and issuance costs to the total does not fix B. Of 560 filers outside
  0.1% with the years one to five, 280 tag such a cost, and it brings 135 within 0.1%.
- D counts a bucket that the filer did not tag as zero. This breaks the rule that a missing bucket
  is not reported.
- E leaves wrong ladders unflagged.
- No other base fits better. For the 899 filers with the years one to five and `LongTermDebt`,
  the sum is within 0.1% for 339 against `LongTermDebt`, 37 of 385 against `LongTermDebt` plus
  `FinanceLeaseLiability`, 81 of 558 against `LongTermDebtNoncurrent` plus `LongTermDebtCurrent`,
  and 13 of 661 against `LongTermDebtNoncurrent`.

## Consequences

- The ladder check has its own tolerance. Every other check keeps ±0.1%.
- A ladder that is off by less than 5% passes, even if one bucket is wrong.
- A filer that tags no "after year five" gets no check. 161 filers tag the fiscal-year buckets of
  the years one to five and `LongTermDebt`, but no "after year five".
- `MaturityMismatch` is a new value of `findings.kind`.

## Open Questions

- A measurement on a later frame reopens the number. A rate of findings far above 9% reopens it.

## Sources

- SEC frames API, `https://data.sec.gov/api/xbrl/frames/us-gaap/<concept>/USD/CY2024Q4I.json`,
  read on 2026-11-15. The concepts: the six buckets and their rolling forms, `LongTermDebt`,
  `LongTermDebtNoncurrent`, `LongTermDebtCurrent`, `FinanceLeaseLiability`,
  `DebtInstrumentUnamortizedDiscountPremiumAndDebtIssuanceCostsNet`,
  `UnamortizedDebtIssuanceExpense`, and `DebtInstrumentUnamortizedDiscount`.
- `xbrl/taxonomy/us-gaap/2026/stm/us-gaap-stm-sfp-dbo1-cal-2026.xml`.
- `xbrl/DESIGN.md`, "Debt Maturities" and "5) Validation".
