# ADR 0022: The Exact Start Splits Copies of One Value Key

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
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
   not compare it with rank 0, reports no `Conflict` for it, and gives it no candidate. The as-is
   layer keeps it. If a rank or rank 0 has no exact start, the two ranks compare as before.
2. **Rank 0 is the fact of the series.** The series holds one value per period end and length in
   quarters, so only one of the facts can enter it. Both parsers rank the copies in the SEC order
   of `iprx`, which puts the length nearer to a full quarter first if the precision is equal.
3. **The Notes parser sets the exact start from `durp`.** The start is the exact end less
   `round((qtrs − durp) × 91.25)` days. The parser builds the duration from the two dates, so the
   length in quarters follows the rule of the instance parser. The instance parser keeps
   `startDate`.
4. **The value key stays as it is:** filing, concept, taxonomy version, exact end, length in
   quarters, unit, dimensions, copy rank. The start is not part of it.
5. **A slot with two exact starts derives nothing and checks nothing.** The slots of
   `core::closed_group` and `core::roll_up_check` stay keyed by the period end and the length in
   quarters. If two values of one slot both have an exact start and the starts differ, the slot
   gives no derived value and no `RollUpMismatch`. A value without an exact start still joins the
   slot of its end and length.

## Options Considered

Measured on 2026-11-15 on the fixture, and on the Notes files `2026_10_notes.zip` and
`2023q1_notes.zip` of the SEC. A conflict key below is a value key with a rank that disagrees with
rank 0 by the rule "Agreement of copies", after the parser drops equal rows.

| Option | Splits conflicts | Changes stored keys | Changes the series and slot keys |
| ------ | ---------------- | ------------------- | -------------------------------- |
| **A (chosen): the exact start tells copies apart** | Fixture: 99 of 99. 2026_10: 329 of 329. 2023q1: 136 of 140 | No. `period_start` gets a value for Notes durations | No. One guard on the slot |
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
| Rank 0 has the length nearest to a full quarter | The conflict keys | 2026_10: 327 of 329. 2023q1: 136 of 140 |

- The 4 conflict keys of 2023q1 that do not split come from one filing. Their two ranks have the
  same length and the same dimensions hash, and they disagree. They stay conflicts.
- The fixture alone gives 3 lengths: 91 days (132 rows), 121 days (197 rows), and 29 days for June
  (44 rows).

## Consequences

- The fixture gives no conflict. Revenue, net income, and the other blocked concepts of the
  quarter get the 3-month value.
- The 4-month stub stays an as-is fact. No standardized fact holds it.
- A concept that the filer reports only for the stub keeps the stub as rank 0. It enters the
  series with 1 quarter. In the fixture, the cash flow lines do this, so the quarter slot holds two
  exact starts, and decision 5 stops its derivations and checks. `Statement::build` already
  rejects facts of two periods.
- `period_start` of `as_is_facts` holds a value for every Notes duration. The Q4 rule and the
  short-year guard then read an exact start for Notes values too.
- The length in quarters of 39 of 4,469,730 Notes durations of 2023q1 changes from `qtrs` to the
  rule of the instance parser. Each spans 20 quarters or more.
- A duration under 46 days has 0 quarters. Two such durations with one end and different starts
  split too. The decision on durations under 46 days can read the exact length in days.

## Open Questions

- Whether the series needs a place for a stub, such as a finding or a length field. The owner
  decides when a consumer asks for stub values.
- Whether a slot with two exact starts reports a finding. Today it reports none.
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
