# ADR 0054: A Duration Under 46 Days Keeps Its Value and Its Own Series

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-28 ([STA-489 comment `aa928dc9`](https://linear.app/state-machine/issue/STA-489/design-decide-how-silver-treats-durations-under-46-days#comment-aa928dc9), accessed 2026-11-28) |
| Date | 2026-11-28 |
| Deciders | The project lead |
| Ticket | [STA-489](https://linear.app/state-machine/issue/STA-489/design-decide-how-silver-treats-durations-under-46-days), accessed 2026-11-28 |

## Context

The length of a duration in quarters is `round(days / 91.3125)`, where `days` is the end less the
exact start. A duration of 45 days or less gets 0 quarters, the same length as an instant. In this
ADR, a **short duration** is such a duration.

The series key of `core/dedup/newest-filing@1` holds the entity, the standard concept, the exact
end, the length in quarters, the unit, and the dimensions. It holds no start. Silver keys a row of
`standardized_facts` on the same fields. So the standardized layer cannot tell a short duration
from an instant, or one short duration from another with the same end.

The measurement of STA-477 over the Notes files of 2023q1 to 2026_10 found 595,875 short durations
in 52,253 filings of 7,517 entities. They are 1.4% of all durations. The most common concepts are
share issues, treasury stock, dividends per share, and monthly rent. Net income appears 730 times.

The 10-Q fixture `0001628280-26-068246` reports June alone, 29 days, next to each quarter. On
`main` at `f3aa37c`, the series step gives 18 short values in 9 standard concepts, such as revenue,
net income, and earnings per share. Each is the only value of its concept at its end, and none gets
a finding.

Three rules treat a short duration like a value of another period:

- `core/dedup/newest-filing@1` puts two short durations of one end into one series, so a value
  for one week can replace a value for one month.
- Chronology fills an expected period with a value of any length whose end lies within 7 days. So a
  value for the last month of a quarter hides a missing quarter.
- `core/dedup/more-precise@1` reports a `Conflict` for a rank with another exact start whose length
  lies nearer to its whole number of quarters than rank 0. For 0 quarters, the shorter duration is
  always nearer. So a one-week rank next to a one-month rank 0 blocks the concept
  ([ADR 0025](0025-exact-start-splits-copies.md), decision 2).

The taxonomy declares one period type per element, so a valid filing cannot report one element as
both an instant and a duration. An alias list maps the concepts of one period type. So a short
duration and an instant meet in one series only through a filing with a period type mismatch.

The decision binds `core::candidate::SeriesKey`, `core::dedup`, `core::chronology`,
`core::required_check`, `core::closed_group`, `core::roll_up_check`, the key of
`standardized_facts`, and the `periods` column of `findings`.
STA-481 and STA-555 follow it.

## Decision

1. **The standardized layer keeps a short duration, with no finding.** Tiers 1 and 2 give it a
   candidate as for any other duration.
2. **The series key of a short duration holds its exact start.** The key of an instant and of a
   duration of 1 quarter or more holds no start. So a short duration never shares a series with an
   instant, or with a short duration of another start. A duration of 1 quarter or more keeps its
   key without the start, so a 52-53 week quarter of two filings stays one series.
3. **A short duration fills only an expected period of 0 quarters.** In chronology, such a period
   comes from a `10-KT` within 45 days of the previous fiscal year end. A short duration fills no
   expected quarter or fiscal year. A blocked short key gives the reason `Conflict` only to a gap
   of 0 quarters. `core::required_check` follows the same rule for the period of report
   ([ADR 0050](0050-missing-required-finding.md), Amendment 1).
4. **In `core/dedup/more-precise@1`, a short rank with another exact start never conflicts by its
   length.** It is another fact and gets no candidate, as ADR 0025 decision 1 says. Rank 0 enters
   the series. [ADR 0025](0025-exact-start-splits-copies.md), Amendment 2, records the change to
   its decision 2.
5. **Two values of one concept with different starts in one slot are mixed starts.**
   `core::closed_group` and `core::roll_up_check` keep their slots on the end and the length in
   quarters, and a slot keeps every value of a concept. A group that holds two values of one
   concept with different starts gives `MixedStarts`, no derived value, and no other check, as for
   two members with different starts ([ADR 0025](0025-exact-start-splits-copies.md), decision 5).
6. **Silver keys a short duration on its start.** The key of a row of `standardized_facts` adds
   `period_start` if `quarters` is 0. The struct of `periods` in `findings` adds a nullable
   `period_start`, set for a period of a short duration and null for `MixedStarts`. The value key and the key of
   `as_is_facts` stay as they are.

## Options Considered

Measured on 2026-11-28 on `main` at `f3aa37c`, and from the STA-477 measurement.

| Option | Short values in standardized | A month replaces another period | Findings | Code |
| ------ | ---------------------------- | ------------------------------- | -------- | ---- |
| **A (chosen): keep, key by the exact start** | 595,875 rows kept | Never | None added | Series key, chronology, one dedup guard, silver key |
| B: keep, key by the period type | 595,875 rows kept | Yes: two short durations of one end share a series | None added | Series key, chronology |
| C: drop | 0 | Never | None added | One filter in tiers 1 and 2 |
| D: keep and flag | 595,875 rows kept | Yes, as today | About 595,875 | A new finding kind |

- A keeps every value and gives each short duration its own series.
- B separates a short duration from an instant only. That case needs a period type mismatch. The
  case of two short durations of one end stays open.
- C loses the June values of the fixture, the 730 net income values, and the value of a `10-KT`
  within 45 days of the previous fiscal year end. Chronology expects a period of 0 quarters for
  such a `10-KT`, so C gives a gap to each duration concept of it.
- D adds a finding to monthly values that are correct, such as rent and dividends, and fixes none
  of the three rules in "Context".

## Consequences

- The fixture keeps its 18 short values. Each has its own series key with its start.
- A value for the last month of a quarter no longer hides a gap of that quarter.
- A short rank with another start no longer blocks the concept of rank 0. The short rank stays an
  as-is fact with no standardized fact.
- Two short values of one concept in one slot give `MixedStarts`, not a sum. The blocked balances
  of `core::roll_up_check` read the keys with no start, not the keys of 0 quarters.
- A 10-Q that tags a concept only for the last month of its quarter gets a `Gap` and a
  `MissingRequired` for the quarter.
- The schema of `findings` changes. STA-620 and STA-621 implement decisions 2 to 6.

## Open Questions

- Whether gold or an API reads a series of short durations, such as monthly rent. The project lead
  decides when a consumer asks for one.
- The decision reopens if a measurement shows short durations that belong in an expected quarter,
  for example a quarter that a filer reports as three months.

## Sources

- [STA-489](https://linear.app/state-machine/issue/STA-489/design-decide-how-silver-treats-durations-under-46-days)
  (accessed 2026-11-28): the counts of the STA-477 measurement by day band and concept.
- [STA-477](https://linear.app/state-machine/issue/STA-477) (accessed 2026-11-28): the
  measurement over the Notes files of 2023q1 to 2026_10.
- `xbrl/src/core/candidate.rs`, `xbrl/src/core/dedup.rs`, and `xbrl/src/core/chronology.rs` at
  `f3aa37c`.
- A scratch test at `f3aa37c` on 2026-11-28: `standardize_series` on the Notes facts of the fixture
  `0001628280-26-068246`, listing the durations of 0 quarters and the findings.
- [ADR 0025](0025-exact-start-splits-copies.md).
