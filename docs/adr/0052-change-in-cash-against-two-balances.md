# ADR 0052: Check of the Change in Cash Against Two Balances

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-27 ([STA-547 comment `e66d8405`](https://linear.app/state-machine/issue/STA-547#comment-e66d8405), accessed 2026-11-27) |
| Date | 2026-11-27 |
| Deciders | The project owner |
| Ticket | [STA-547](https://linear.app/state-machine/issue/STA-547), accessed 2026-11-27 |

## Context

A cash flow statement ends with the change in cash. The change ties the cash at the start of the
period to the cash at the end: opening balance + change = closing balance. The checks of the
roll-up tree compare values of one period only, so none of them ties the statement to the balances.

The facts this check builds on:

- Each value of `NetChangeInCash` has a scope. No scope is the full scope: cash, cash equivalents,
  and restricted cash (ASU 2016-18). The scope `ExcludesRestrictedCash` marks a value from a concept
  before ASU 2016-18, such as `CashAndCashEquivalentsPeriodIncreaseDecrease`.
- The standard concept `CashAndRestrictedCash` holds the cash with restricted cash at an instant.
  Only the ASU 2016-18 concepts fill it, so a period before ASU 2016-18 has no value.
- The balance sheet concept `CashAndEquivalents` holds cash and cash equivalents without
  restricted cash. Its alias is `CashAndCashEquivalentsAtCarryingValue`, and `Cash` is a part.
  `CashAndDueFromBanks` has no mapping, so a bank before 2018 has no value.
- Every duration has an exact start ([ADR 0025](0025-exact-start-splits-copies.md), Amendment 1).
- No rule derives `NetChangeInCash` from two balances. The difference of two balances is the input
  of this check only.

The decision binds the series step (`check_roll_ups`), the silver table `findings`, and the
coverage report that counts findings.

In this document, the **opening balance** is the balance at the day before the exact start of the
change. The **closing balance** is the balance at the end of the change.

## Decision

1. **The scope of the change picks the balance.** A change with no scope reads
   `CashAndRestrictedCash`. A change with the scope `ExcludesRestrictedCash` reads
   `CashAndEquivalents`.
2. **The opening balance sits at the day before the exact start.** The check reads the balance
   whose instant equals that day, with no window. A year-to-date change of a 10-Q starts on the
   first day of the fiscal year, so its opening balance is the balance at the end of the prior
   fiscal year. A quarter, a derived Q4, and a stub period follow the same rule.
3. **The check compares values without dimensions in one unit.** It reads the series after dedup
   and the period identities, so each balance can come from any filing of the entity. A derived
   change, from Q4 or from `net-change-in-cash`, gets the check too.
4. **The tolerance is ±0.1% of the largest absolute value of the opening balance, the change, and
   the closing balance.** It is the tolerance of the closed groups.
5. **A failed check gives one `RollForwardMismatch` finding.** `concepts` holds the balance concept,
   then `NetChangeInCash`. `periods` holds the period of the change. `reason` holds `cash`, the
   name of the roll forward. `confidence` holds the weakest confidence of the three inputs.
6. **A missing input gives no check and no finding.** The check skips a period if the change, the
   opening balance, or the closing balance has no value, or if dedup blocks one of them. A value
   with no exact start also gets no check.

## Options Considered

Measured on 2026-11-27 on `main` at `072e327` with a local script. The script read the Notes rows
of each 10-K and 10-Q, original and amended, and applied the alias and part rules of the four
concepts, with the narrow scope concepts of the context. It compared the balances of the same
filing only. The script was not committed.

- `2026_10`: the Notes file of October 2026. 1,015 filings of 991 filers.
- The fixture set: the 105 Notes filings of `xbrl/src/tests/fixtures/data/`.

### The Balance for the Narrow Scope

The fixture set has 93 changes with the scope `ExcludesRestrictedCash`, all before 2018.

| Option | Pass | Fail | No check |
| ------ | ---- | ---- | -------- |
| **A. `CashAndEquivalents` (chosen)** | 83 | 3 | 7 |
| B. No check for the narrow scope | 0 | 0 | 93 |

- A checks 86 of 93 changes. The 3 failures are false: the statement reconciles
  `CashAndCashEquivalentsAtCarryingValueIncludingDiscontinuedOperations`, a wider balance. They are
  Air Products fiscal year 2012 (421.4 + 31.9 = 453.3 against 454.4 million USD) and Ameren 2012
  and 2013.
- Of the 7 changes with no check, 3 are JPMorgan Chase 2011 to 2013. The bank reconciles
  `CashAndDueFromBanks`, for example 53,723 − 13,952 = 39,771 million USD in 2013, and
  `CashAndEquivalents` has no value. The other 4 lack an opening balance in the same filing.
- B gives no false finding, but no change before 2018 gets a check.

### The Finding

| Option | Kind in silver | `concepts` |
| ------ | -------------- | ---------- |
| **A. New kind `RollForwardMismatch` (chosen)** | One new value | The balance, then the change |
| B. `RollUpMismatch` with the reason `cash-roll-forward` | No new value | One concept at two dates |

- A keeps the meaning of `RollUpMismatch`: a sum of one period. A later roll forward, such as one
  per component of equity, takes the same kind with its own name in `reason`.
- B mixes two shapes in one kind, so a reader of `RollUpMismatch` must check `reason` first.

### The Result on `2026_10`

Full scope only: the file has no change with the narrow scope.

| Filers | Count |
| ------ | ----- |
| Every checked change passes | 891 |
| Every checked change passes, and a change lacks a balance | 9 |
| A change fails | 20 |
| Every change lacks a balance | 15 |
| No change in cash | 56 |

- Per change: 1,877 pass, 24 fail, and 37 lack a balance. Of the 1,901 checked changes, 1,861 tie
  exactly, 16 within 0.1%, 8 between 0.1% and 1%, and 16 above 1%.
- Restaurant Brands International shows a failure the check is for. Its total change of −180
  million USD holds 9 million of cash held for sale, and its balance moves by −189.
- No change lacks a balance because the instant lies a few days off. No instant within 7 days of
  the day before the exact start was found where the exact day had none.

The fixture set adds 166 passes, 3 failures, and 4 changes without a balance for the full scope.
The 3 failures are Crown Castle 2023 to 2025: the filing tags
`CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseIncludingExchangeRateEffect`
as 1, −1, and 0 million USD, while its balance moves by −46, 14, and 13.

## Consequences

- The statement of cash flows ties to the balances for each period with three values.
- Silver gains one finding kind. The key of a finding stays as it is.
- A change with the narrow scope and cash of discontinued operations gives a false finding. It was
  3 of 86 checked changes in the fixture set.
- A bank before 2018 gets no check: `CashAndEquivalents` has no value.
- The check reads balances from every filing of the entity. The counts above read one filing, so
  the series has fewer changes without a balance.
- Not covered: a change in a unit other than the unit of the balances, and values with dimensions.

## Open Questions

- A balance for cash of discontinued operations before 2018, such as
  `CashAndCashEquivalentsAtCarryingValueIncludingDiscontinuedOperations`, would remove the 3 false
  findings. It needs its own scope or concept. A new variant of the scope reopens decision 1.
- The rule along the calculation tree that adds `CashAndDueFromBanks` and
  `InterestBearingDepositsInBanks` would give banks a balance. It reopens decision 1 for banks.
- The 24 failures of `2026_10` are not classified. If most of them come from one cause in the
  alias lists, that cause reopens decision 1 or 4.

## Sources

- `2026_10_notes.zip` ([Financial Statement and Notes Data Sets](https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2026_10_notes.zip),
  accessed 2026-11-27): the rows of `2026_10`.
- `xbrl/src/tests/fixtures/data/` at `072e327`: the Notes rows of the fixture set.
- `xbrl/src/us_gaap/mappings.rs` at `072e327`: the alias and part rules of `CashAndEquivalents`
  and `NetChangeInCash`.
- [ADR 0025](0025-exact-start-splits-copies.md), Amendment 1: the exact start of every duration.
- [STA-541 comment `d2a35b0a`](https://linear.app/state-machine/issue/STA-541#comment-d2a35b0a),
  accessed 2026-11-27: the split of the scope design and this check.
- [STA-547 comment `e66d8405`](https://linear.app/state-machine/issue/STA-547#comment-e66d8405),
  accessed 2026-11-27: the questions and the answers of the owner.
