# ADR 0050: A Finding per Filing for a Missing Required Concept

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-27 ([STA-542 comment `94d0e5aa`](https://linear.app/state-machine/issue/STA-542#comment-94d0e5aa), accessed 2026-11-27). The project owner confirmed the consequence for IFRS filers on 2026-11-27 ([STA-542 comment `f0d34e74`](https://linear.app/state-machine/issue/STA-542#comment-f0d34e74), accessed 2026-11-27) |
| Date | 2026-11-27 |
| Deciders | The project owner |
| Ticket | [STA-542](https://linear.app/state-machine/issue/STA-542), accessed 2026-11-27 |

## Context

The lead decided on 2026-10-12 that the roll-ups of the MVP Decision Spike are required, and that a
missing roll-up shows "missing" with a reason. 11 of them map to a standard concept. The flag
`required` of `xbrl/src/us_gaap/mappings.rs` marks these 11.

The check runs in `standardize_series`, after Tier 3, on the values of one entity. Silver stores
its output as rows of `findings`. The coverage report of a run counts these rows. So the shape of
the finding binds the series step, the silver schema, and the coverage report.

In this document, a **periodic report** is an annual report (`10-K`, `10-KT`, `20-F`, `40-F`) or a
quarterly report (`10-Q`, `10-QT`) that is no amendment. Its **period of report** is the end date
that `Filing::period_of_report` holds.

## Decision

1. **One `MissingRequired` finding per periodic report and required concept without a value.** The
   finding holds the entity, the filing, the standard concept, and the period of report.
2. **A value of any length fills the period of report.** A value fills it if it has no dimensions
   and its end lies within 7 days of the period of report, as in the gap check. Amendments get no
   finding, because their original report covers the same period.
3. **The reason is `Conflict`, `Untagged`, or none.** `Conflict`: dedup blocks a value of the
   concept at the period of report. `Untagged`: the filing has no candidate of the concept, without
   dimensions, for any period. Otherwise the reason is none. No other cause is guessed.
4. **Silver stores the finding as one row of `findings`.** `kind` is `MissingRequired`. `concepts`
   holds the standard concept. `periods` holds one entry: the period of report with `quarters` 0,
   because no length is expected. `filing_id` holds the filing. `reason` holds the reason or null.

## Options Considered

Measured on 2026-11-27 on `main` at `e398432` with the change of this ADR, on the 14 fixture 10-Ks
of the acceptance test A2 (`xbrl/tests/acceptance_open_roll_ups.rs`).

### Which Periods the Check Covers

| Option | Findings on the 14 10-Ks | Needs |
| ------ | ------------------------ | ----- |
| **A. Period of report, any length (chosen)** | 21, each an open roll-up of A2 | The filing and its period of report |
| B. Period of report, exact length | 21 on these 10-Ks | The fiscal period of a 10-Q to know its length, and the length of a `10-KT` |
| C. Every expected period of the gap check | Not measured | The calendar of the gap check |

- A gives the same 21 findings as the open roll-ups of A2.
- B must know that the cash flow of a `10-Q` covers the year to date, not the quarter. A `10-Q`
  without a fiscal period gets no length.
- C reports again what `Gap` reports, and it reports concepts for periods before the first filing
  of the entity.

### The Length in Silver

| Option | Join with `standardized_facts` on `periods` |
| ------ | ------------------------------------------- |
| **A. `quarters` 0 (chosen)** | Matches no duration. A reader joins on `filing_id` and the concept |
| B. The length of the fiscal period of the filing | Needs the length of option B above |

## Consequences

- The coverage report counts the rows of `MissingRequired` by `filing_id` and concept.
- A filing whose only value of a concept has another length, such as a Q4 value without a fiscal
  year value, gets no finding.
- A filing with no period of report gets no finding.
- A `20-F` or `40-F` in `ifrs-full` gets one `Untagged` finding per required concept, because no
  plug-in maps IFRS concepts. Such a row says that the concept has no mapping, not that the filer
  left out the statement.
- The parse of the `sec` crate keeps its own check: it reads the array `REQUIRED_CONCEPTS`, not
  the flag. Its set differs from the flag in four concepts.

## Open Questions

- The cash at the end of the period has no standard concept until
  [STA-541](https://linear.app/state-machine/issue/STA-541), accessed 2026-11-27, decides. A new
  required concept adds rows and needs no change of this shape.
- If the coverage report needs the length of the missing period, option B of "The Length in
  Silver" reopens decision 4.

## Sources

- `xbrl/tests/missing_required.rs` at the commit of this ADR: the 21 findings on the 14 10-Ks.
- `xbrl/tests/acceptance_open_roll_ups.rs` at `e398432`: the open roll-ups of the 14 10-Ks.
- [STA-542](https://linear.app/state-machine/issue/STA-542), accessed 2026-11-27: the decision of
  the lead of 2026-10-12, and the Definition of Done.
- The owner chose the 11 roll-ups as the required set, the periodic reports as the scope, and the
  flag in place of the array in the session of 2026-11-27.
