# ADR 0025: The Exact Start Splits Copies of One Value Key

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([STA-490 comment `4538cd45`](https://linear.app/state-machine/issue/STA-490/design-split-a-conflict-of-two-lengths-that-share-a-value-key#comment-4538cd45)). Amendment 1: Accepted by the project lead on 2026-11-21 ([STA-546 comment `901891a8`](https://linear.app/state-machine/issue/STA-546/refactor-make-the-exact-start-of-a-duration-required#comment-901891a8)), and in its final wording ([STA-546 comment `9a7200b6`](https://linear.app/state-machine/issue/STA-546/refactor-make-the-exact-start-of-a-duration-required#comment-9a7200b6)). |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-490](https://linear.app/state-machine/issue/STA-490/design-split-a-conflict-of-two-lengths-that-share-a-value-key) |

## Context

The value key of an as-is fact holds the exact period end and the length in quarters, not the
start (`xbrl/DESIGN.md`, "Value Key"). The length in quarters is `round(days / 91.3125)`. So a
4-month stub and a 3-month quarter with the same end both have 1 quarter, and they share a value
key. The parser numbers them as two copies, rank 0 and rank 1.

`core/dedup/more-precise@1` compares every rank with rank 0. If the values disagree, it reports a
`Conflict`. If tiers 1 and 2 pick the concept, the conflict blocks the candidate of the filing, and
the standard concept has no value for that period.

The 10-Q fixture `0001628280-26-068246` reports a quarter from 1 July and a period of four months
from 1 June, both to 30 September, for 99 value keys. All 99 are conflicts, and they block revenue,
net income, and more for the quarter. The measurement for the "Suspicious Periods" section of
`xbrl/DESIGN.md` found 3,291 such keys in 510 filings of 311 entities over the Notes files of
2023q1 to 2026_10.

Terms in this ADR:

- **Exact start:** the first day of a duration. The instance gives it as `startDate`.
- **`durp`:** a column of the Notes `num.tsv` file. It is `qtrs` less the true length in quarters.
  The SEC description of the field gives the opposite sign.
- **Copy:** a rank of a value key that reports the same period as rank 0.

The decision binds the Notes parser (`xbrl/src/adapter/edgar/notes/`), `core::dedup`,
`core::closed_group`, `core::roll_up_check`, and the `period_start` column of `as_is_facts`.

## Decision

1. **Two ranks of one value key are copies only if their exact starts are equal.** A rank whose
   exact start differs from the start of rank 0 is another fact. `core/dedup/more-precise@1` does
   not compare it with rank 0 or with another rank, and gives it no candidate. It reports no
   `Conflict` for it, except by decision 2. The as-is layer keeps it. If a rank or rank 0 has no exact start, the two ranks
   compare by value, as copies.
2. **Rank 0 is the fact of the series, unless a stub would take the place of the quarter.** The
   series holds one value per period end and length in quarters, so only one of the facts can
   enter it. The parsers rank by precision first, and `adapter::edgar` ranks in the order of the
   JSON. So rank 0 can be the stub. If a rank with another exact start has a length nearer to its
   whole number of quarters than rank 0 (`Period::distance_to_full_quarters`), the rule reports a
   `Conflict`, as for copies that disagree.
3. **The Notes parser sets the exact start from `durp`.** The start is the exact end less
   `round((qtrs − durp) × 91.25)` days. The parser builds the duration from the two dates, so the
   length in quarters follows the rule of the instance parser. The instance parser keeps
   `startDate`.
4. **The value key stays as it is:** filing, concept, taxonomy version, exact end, length in
   quarters, unit, dimensions, copy rank. The start is not part of it.
5. **A group with two exact starts derives nothing, checks nothing, gets no other line, and
   reports it.** The slots
   of `core::closed_group` and `core::roll_up_check` stay keyed by the period end and the length
   in quarters. A slot holds every group of its period. The members of a group are the
   values that the statement counts in it: the start of a step, the members that stand for a
   start with no value, and the children of a transparent parent that join the group. If two members of one group in a slot both have an exact start and the starts differ, that group gives
   no derived value, no `RollUpMismatch`, and no `NegativeOther`. It reports a `MixedStarts`
   finding, so a reader sees that the group has no check. `Statement::build` accepts facts whose
   periods differ only in the start, and gives such a group no other line. The other groups of
   the slot keep their checks and their other lines. In `findings`,
   `concepts` holds the members of the group with a value, in the order of the roll-up tables,
   `periods` holds the period end and length of the slot, and `reason` holds the name of the
   group. A value without an exact start joins the slot of its end and length.

## Amendment 1: Every Duration Has an Exact Start

Date: 2026-11-21.

**Reason.** ironcapitaleu/arkad#606 put decision 3 on `main`. So every parser gives each
duration an exact start: the Notes parser from `durp`, the instance parser from `startDate`, and
`adapter::edgar` from the `start` field of the JSON. If the field is missing, `adapter::edgar`
builds an instant. No code builds a duration without a start, except tests. Decisions 1 and 5 still name
a rank or a value without an exact start. That case cannot occur, and its branches hide a source
that drops the start.

**Decision.** A duration always has an exact start. `Duration` holds the start as a date, not as
an option, and the constructor from an end and a length in quarters is gone. In decision 1, two
ranks compare as copies if they have the same start, or if either rank is an instant. In
decision 5, a value without an exact start does not exist, so the sentence on it no longer
applies. An instant has no start, as before.

**Consequences.**

- A parser cannot build a duration without a start. If a source gives none, the parser must
  reject the fact. No rule silently treats it as a copy or lets it join every slot.
- `core/period/q4-from-fy@1` decides a short year by the day count alone. Its fallback on a
  `10-KT` and on the previous annual report is gone, so `derive_q4` takes no filings.
- `StartBeforeEarliestYear` reads the exact start. Its estimate from the length is gone.
- The `start` of a duration in the `/v1` API contract is a required date, not a nullable one.
- `period_start` of `as_is_facts` stays nullable, because an instant has no start.

## Options Considered

Measured on 2026-11-15 on the fixture, and on the Notes files `2026_10_notes.zip` and
`2023q1_notes.zip` of the SEC. A conflict key below is a value key with a rank that disagrees with
rank 0 by the rule "Agreement of copies", after the parser drops equal rows.

| Option | Splits conflicts | Changes stored keys | Changes the series and slot keys |
| ------ | ---------------- | ------------------- | -------------------------------- |
| **A (chosen): the exact start tells copies apart** | Fixture: 99 of 99. 2026_10: 329 of 329. 2023q1: 136 of 140 | No. `period_start` gets a value for Notes durations | No. One guard per group of the roll-up tree |
| B: the exact start in the value key | The same as A | Yes: every key of a duration | Yes, or the series and slots of one period split by start |
| C: keep the block | None | No | No |

- A splits every conflict that B splits, and keeps the keys of `as_is_facts`, `candidates`, and
  `standardized_facts`.
- B changes the key that the owner fixed on 2026-11-07: the value key holds the period end and the
  length, not the start. The series keys, the slots, and the derived Q4 would need the start too.
- C leaves the quarter without a value in each filing that reports a stub next to it.

How the sources agree:

| Check | Data | Result |
| ----- | ---- | ------ |
| `(qtrs − durp) × 91.25` is a whole number of days | 817,516 durations of 2026_10, 4,469,730 of 2023q1 | Every one, within 0.01 days |
| The start from `durp` equals the start of an instance fact with the same concept and end | The fixture and the 13 other filings of 2026_10 with a conflict: 7,173 Notes durations | 7,173 match, 0 differ |
| `round(days / 91.3125)` equals `qtrs` | The durations of 2023q1 | All but 39. Each of the 39 spans 20 quarters or more |
| Rank 0 has the length nearest to its whole number of quarters | The conflict keys | Fixture: 99 of 99. 2026_10: 327 of 329. 2023q1: 136 of 140. By decision 2, the others stay conflicts |

- The 4 conflict keys of 2023q1 that do not split come from one filing. Their two ranks have the
  same length and the same dimensions hash, and they disagree. They stay conflicts.
- The fixture alone gives 3 lengths: 91 days (132 rows), 121 days (197 rows), and 29 days for June
  (44 rows).

## Consequences

- The fixture gives no conflict. Revenue, net income, and the other blocked concepts of the
  quarter get the 3-month value.
- The 4-month stub stays an as-is fact. No standardized fact holds it.
- A concept that the filer reports only for the stub keeps the stub as rank 0. It enters the
  series with 1 quarter. In the fixture, the cash flow lines do this. The group
  `operating-cash-flow` starts from the 3-month `NetIncome` and holds 4-month lines. It is an open
  step with no check and no derived value, so decision 5 reports `MixedStarts` and drops its other
  line from the cash flow statement. `net-change-in-cash` holds only 4-month values, and
  the income statement groups hold only 3-month values, so they keep their checks.
- The roll-up check gives each fact the period of its slot before it builds a statement, and
  `Statement::build` accepts periods that differ only in the start. So decision 5 is the only
  guard against two starts, in the checks and in the statement output.
- `period_start` of `as_is_facts` holds a value for every Notes duration. The Q4 rule and the
  short-year guard then read an exact start for Notes values too.
- The length in quarters of 39 of 4,469,730 Notes durations of 2023q1 changes from `qtrs` to the
  rule of the instance parser. Each spans 20 quarters or more.
- A duration under 46 days has 0 quarters. Two such durations with one end and different starts
  split too. The decision on durations under 46 days can read the exact length in days.

## Open Questions

- Whether the series needs a place for a stub, such as a length field. The owner decides when a
  consumer asks for stub values.
- A slot that holds only stubs has one exact start, so decision 5 does not fire. Its values enter
  the series with the rounded length in quarters. The decision on lengths that do not fit the
  quarter grid covers this case.
- The decision reopens if a Notes file gives a `durp` that is not a whole number of days, or a
  start that differs from the instance.

## Sources

- Fixture: `xbrl/src/tests/fixtures/data/notes/0001628280-26-068246/num.tsv` and
  `xbrl/src/tests/fixtures/data/instance/0001628280-26-068246/fdxf-20260930_htm.xml`, read on
  2026-11-15.
- `https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2026_10_notes.zip` and
  `2023q1_notes.zip`, downloaded on 2026-11-15.
- The instance documents of the 13 filings of 2026_10 with a conflict, from the `index.json` of
  each filing folder on EDGAR, downloaded on 2026-11-15.
- Scripts: Python over `num.tsv`. They group rows by the value key, drop equal rows, compare the
  ranks by "Agreement of copies", and compute `(qtrs − durp) × 91.25`.
- `xbrl/DESIGN.md`, "Value Key", "Dedup Rules", and "Suspicious Periods".
