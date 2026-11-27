# ADR 0041: One Value per Filing Between the Transform States

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project owner confirms |
| Date | 2026-11-21 |
| Deciders | The project owner |
| Ticket | [STA-332](https://linear.app/state-machine/issue/STA-332), accessed 2026-11-27 |

## Context

A transform task turns one unit into silver rows: one Notes file, or the instances of one filed
date. The task runner runs one state machine per filing on N worker threads, and hands the rows of
whole filings to one writer ([ADR 0035](0035-one-task-per-notes-file.md) keeps one task per Notes
file).

A Notes file is large and a filing is small. Measured on 2026-11-07 in `2025q3_notes.zip`:
`num.tsv` holds 1,188.6 MB of text and 7,697,635 rows. One filing holds 785 rows of `num.tsv` at the
median and 27,529 at most. A pass that cuts `num.tsv` at each change of `adsh` peaked at 36 MB.

STA-329 asks whether a state can output a stream of records that the next state consumes. It
names the Notes backfill as the case. This choice binds STA-329 and every FEATURE ticket of the
filing states.

## Decision

1. **Each state of the filing machine outputs one value for one whole filing.** No state of the
   transform outputs a stream of records.
2. **Only the unit reader streams.** It runs outside the state machine, on one thread of the task,
   and sends whole filings to the workers through a bounded channel.
3. **The task runner drives each machine by its transitions,** not by its event stream. No row of
   a filing enters event JSON.

## Options Considered

Measured on 2026-11-08 in a container with 4 cores and 15 GB of memory, on one thread (Linear
document "STA-424 Findings: Silver Parquet Layer and Polars for the Transform Step").

| Option | Peak memory, `2024q1` | New framework code | Per-filing errors |
| ------ | --------------------- | ------------------ | ----------------- |
| **A. One value per filing, the reader streams filings (chosen)** | 463 MB for the whole unit with the stream split | None | Typed per filing, from each state |
| B. Streamable states: the split, parse, and write states pass records | Not measured. Bounded by the channels, as A | A stream type for state data, its trait bounds, backpressure, and the graph edge (STA-329) | A bad record mid-stream needs a new rule |
| C. One value per unit: a state holds the whole Notes file | 4,448 MB for an eager read of `2024q1` | None | Typed per unit only |

- A streams where the data is large, between the unit and the filings. Inside a filing, the data
  is small, so one value costs no memory that matters.
- B gives the same memory bound as A, and needs the framework design of STA-329 first.
- C breaks the memory of a task: the job design gives a Notes task 2 GiB.

## Consequences

- The MVP needs neither streamable states nor fork and join. STA-329 stays a design for later
  work.
- Each filing has its own typed outcome. So one bad filing leaves its unit, and the rest goes on.
- The reader holds no state of the machine. Its errors fail the unit, not a filing.
- A filing much larger than 27,529 rows raises the peak of a worker. Not covered: a filing that
  alone passes the memory of a task.

## Open Questions

- A filing larger than 100,000 rows in `num.tsv` reopens this decision.
- A source whose unit is one huge document, such as one instance with millions of facts, reopens
  it.

## Sources

- Linear document "STA-424 Findings: Silver Parquet Layer and Polars for the Transform Step",
  2026-11-08: the peak of the stream split and of the eager read of `2024q1`.
- [Linear document "STA-425 Findings: Data Model and Processing of the Silver Layer"](https://linear.app/state-machine/document/sta-425-findings-data-model-and-processing-of-the-silver-layer-2cfdecb2c70a),
  accessed 2026-11-21, section 8, 2026-11-07: the size of `2025q3`, rows per filing, and the peak
  of a streamed pass.
- [STA-329](https://linear.app/state-machine/issue/STA-329), accessed 2026-11-21, "Additional
  Notes": the idea of streamable states.
- [ADR 0035](0035-one-task-per-notes-file.md).
