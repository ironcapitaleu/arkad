# ADR 0042: A Rejected Filing Leaves Its Unit, and Too Many Fail the Unit

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-27 ([STA-332 comment `f86f6786`](https://linear.app/state-machine/issue/STA-332#comment-f86f6786), accessed 2026-11-27). Amendment 1: Accepted by the project owner on 2026-11-28 ([STA-578 comment `d54571b7`](https://linear.app/state-machine/issue/STA-578#comment-d54571b7), accessed 2026-11-28) |
| Date | 2026-11-21 |
| Deciders | The project owner |
| Ticket | [STA-332](https://linear.app/state-machine/issue/STA-332), accessed 2026-11-27 |

## Context

A transform unit holds many filings: every selected filing of one Notes file, or of one filed
date. A parser of `xbrl::adapter::edgar` can reject one filing as a whole:

- `notes::parse` returns `InvalidSubmissionRow` if the `sub` row breaks a rule of a core type, for
  example a fiscal year end that is not a month and a day.
- `instance::parse` returns `InvalidXml` if the text is not XML, and `InvalidInstanceFiling` if a
  cover fact is missing or breaks a rule.

The failure table of the job design (STA-463, ironcapitaleu/arkad#603) already writes the unit
without a rejected filing, and counts the filing in `dropped_rows` with the reason
`rejected_filing`. It leaves two points to STA-332: which filing is rejected, and the share of
rejected filings that fails a unit. `unit_manifests` has no `rejected` column.

A unit that fails stays in the next plan, so a unit that always fails runs again in each run. A
unit that writes without its rejected filings keeps them out of silver until the unit is written
again.

Facts for the size of a unit, measured on 2026-11-07 and 2026-11-08:

| Unit | Selected filings |
| ---- | ---------------- |
| Notes file `2024q1` | 5,762 |
| Notes file `2025q3` | 5,731 |
| Filing day 2026-10-28 | 167 instances |
| A business day of the daily path | About 90 (estimate) |

The prototype findings report no rejected filing in `2024q1`, `2012q4`, `2026_10`, and the day
2026-10-28. No measurement gives the share of rejected filings over the whole history.

This choice binds the filing step, the silver writer, and the run summary of the transform.

## Decision

1. **A filing is rejected when its filing machine returns an error.** Each such error comes from
   the data of that one filing. The machine does no I/O, so a transient error never rejects a
   filing. A transient error or a failed check of the container or the tables fails the unit.
2. **A rejected filing leaves the unit, and the unit goes on.** The filing writes no row.
   `dropped_rows` counts every source row of the filing with the reason `rejected_filing`, so the
   row count check of silver still holds. The task logs one `filing_rejected` event as `warn`.
3. **The unit fails if more than 5% of its selected filings are rejected and at least 3 are, or if
   every selected filing is rejected.**
   The writer checks this after the last filing of the unit, before it writes the `unit_manifests`
   row. A unit that fails this way is a bad unit in the failure table of the job design.

## Amendment 1: The Reason Text `RejectedFiling`

Date: 2026-11-28. Ticket: [STA-578](https://linear.app/state-machine/issue/STA-578/feature-buildfilingrows-run-filing-and-filingoutcome-of-the-filing), accessed 2026-11-28.

### Reason

`dropped_rows` of `unit_manifests` stores each reason as text. On `main` at `083de59`
(2026-11-28), `DropReason` of `sec` prints `UnmappedRow` and `DuplicateRow`, as ADR 0053 sets
them. Decision 2 above sets `rejected_filing`. So one column held two styles, and a query on
`reason` had to know the style of each reason. `UnmappedRow` is also a `kind` of `findings`, in
the same PascalCase. No silver data exists on 2026-11-28, so a rename costs no migration.

### Decision

1. **The reason of a rejected filing is `RejectedFiling`.** It replaces `rejected_filing` in
   decision 2.
2. **Every reason of `dropped_rows` is the PascalCase name of its `DropReason` variant.** A new
   reason follows this rule, for example `FormNotSelected` and `NoSubRow` of the Notes reader.

### Consequences

- A query on `dropped_rows.reason` uses one style for every reason.
- The text of a reason equals the name of its Rust variant, so a reader finds the variant by the
  text.
- Not covered: the `kind` values of `findings`. They are PascalCase already.

## Options Considered

| Option | One bad filing in a day of 10 | A parser change that rejects most 10-Q filings |
| ------ | ----------------------------- | ---------------------------------------------- |
| **A. 5% and a floor of 3 (chosen)** | The day is written without it | The unit fails |
| B. 5% alone | The day fails in each run, and its 9 other filings stay out of silver | The unit fails |
| C. No threshold | The day is written without it | Every unit is written without most filings. Only the run summary shows it |
| D. Any rejected filing fails the unit | The day fails in each run | The unit fails |

- A fails the unit if a change breaks a whole form, or every filing of a small unit. If a unit has
  one bad filing, A writes its other filings.
- B and D let one filing block every other filing of its unit until a fix.
- C writes the damage of a broken parser to silver as a valid unit.

## Consequences

- A rejected filing is visible in the logs and the run summary of the run that wrote its unit. A
  later run that skips the unit by rule 2 does not report it again.
- A filing stays out of silver until its unit is written again: new raw bytes, or a new
  `transform_version` after a parser fix.
- The threshold counts per unit. So a Notes file needs 289 rejected filings to fail, and a day of
  90 filings needs 5.
- Not covered: a filing whose rows the parser maps without an error but with wrong values. The
  golden test and the data checks find those.

## Open Questions

- The 5% and the floor of 3 rest on no measurement of rejected filings. The first full backfill
  measures the share per unit. A unit that fails by this rule in that backfill, or a share above
  1% in any unit, reopens the decision.
- The design of failure modes and dead letters decides when a unit that keeps failing leaves the
  plan.

## Sources

- `xbrl/src/adapter/edgar/notes/mod.rs` and `xbrl/src/adapter/edgar/instance/mod.rs` on `main` at
  `7e2c83e`: the errors of each parser.
- Linear document "STA-424 Findings: Silver Parquet Layer and Polars for the Transform Step",
  2026-11-08: the filings of `2024q1` and of the filing day 2026-10-28, and the `dropped_rows`
  reasons of the prototype.
- [Linear document "STA-425 Findings: Data Model and Processing of the Silver Layer"](https://linear.app/state-machine/document/sta-425-findings-data-model-and-processing-of-the-silver-layer-2cfdecb2c70a),
  accessed 2026-11-21, section 6, 2026-11-07: the 5,731 filings of the four forms in `2025q3`.
- [STA-332](https://linear.app/state-machine/issue/STA-332), accessed 2026-11-27, "Definition of
  Done": the proposal of
  STA-425 to skip a rejected filing and fail a unit above about 5%.
