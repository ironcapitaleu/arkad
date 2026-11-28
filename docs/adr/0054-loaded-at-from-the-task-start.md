# ADR 0054: `loaded_at` from the Start of the Task

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-28 ([STA-578 comment `5008e1e2`](https://linear.app/state-machine/issue/STA-578#comment-5008e1e2), accessed 2026-11-28) |
| Date | 2026-11-28 |
| Deciders | The project lead |
| Ticket | [STA-578](https://linear.app/state-machine/issue/STA-578/feature-buildfilingrows-run-filing-and-filingoutcome-of-the-filing) |

## Context

Every row of `as_is_facts`, `filings`, `candidates`, and `findings` ends with the run columns
`transform_version`, `execution_id`, and `loaded_at`. The row types hold them as `RunColumns`.

`BuildFilingRows`, the last state of the filing machine, builds these rows. Its context,
`FilingContext`, held the registry, `transform_version`, and `execution_id` on `main` at `2f8872d`
(2026-11-28). It held no time. So the state had no `loaded_at` to write.

The docs of `RunColumns` on the same commit defined `loaded_at` as "The time of the write". The writer
writes a unit after the last filing of the unit. The filing machine runs before the write and does
no I/O.

The choice binds the task runner of STA-558, which builds the context, and every test of a state
that builds rows.

## Decision

1. **`FilingContext` holds `loaded_at`, the start time of the task.** The task runner sets it once,
   when the task starts. `BuildFilingRows` writes it to every row of every filing of the task.
2. **`loaded_at` means the start time of the task that wrote the row.** The silver contract and
   the docs of `RunColumns` say so.

## Options Considered

Decided on 2026-11-28 by the project lead, in the session of STA-578.

| Option | Rows of one task | A test of the state | Change to the context |
| ------ | ---------------- | ------------------- | --------------------- |
| **A (chosen): a field of `FilingContext`, set at the start of the task** | One time | Compares whole rows | One field |
| B: `Timestamp::now()` in `BuildFilingRows` | One time per filing | Cannot compare `loaded_at` | None |
| C: the writer sets `loaded_at` at the write | One time per unit | Compares whole rows, with a placeholder time | None, but the rows hold a placeholder until the write |

- A keeps the filing machine free of the clock, as it is free of I/O. A test gives fixed rows.
- B reads the clock in a state, so two runs of one filing give different rows.
- C needs a placeholder in each row, and the writer must change every row before the write.

## Consequences

- Every row of one task carries the same `loaded_at`, earlier than the write of its unit by up to
  the run time of the task.
- `loaded_at` stays out of `content_digest`, so the earlier time changes no digest.
- The task runner of STA-558 must set `loaded_at` when it builds the context.

## Open Questions

- None.

## Sources

- `sec/src/lib/shared/filing_context/mod.rs` and `sec/src/lib/datasets/implementations/run_columns/mod.rs`
  on `main` at `2f8872d`, 2026-11-28.
- The answer of the project lead to the question "Where does `BuildFilingRows` take `loaded_at`
  from?", 2026-11-28: "Field on `FilingContext`".
