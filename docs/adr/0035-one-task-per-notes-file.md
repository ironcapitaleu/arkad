# ADR 0035: One Task Transforms One Whole Notes File

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-21 ([STA-463 comment `5fa1a8f7`](https://linear.app/state-machine/issue/STA-463#comment-5fa1a8f7)). Amendment 1: Accepted by the project owner on 2026-11-21 ([STA-463 comment `84a54644`](https://linear.app/state-machine/issue/STA-463#comment-84a54644)) |
| Date | 2026-11-16 |
| Deciders | The project owner |
| Ticket | [STA-463](https://linear.app/state-machine/issue/STA-463/design-silver-writer-and-transform-job-plan) |

## Context

A `notes_file` unit holds every filing of one Notes file. STA-463 sets a task timeout of one
hour and a target of 15 minutes for the largest Notes file, so the timeout is 4 times the target
(STA-463, Definition of Done). If a file breaks the target, the transform must split it across
tasks, and those tasks must write one `silver_files` row.

Measured on 2026-11-08 in a container with 4 cores and 15 GB of memory, on one thread, from local
disk (Linear document "STA-424 Findings: Silver Parquet Layer and Polars for the Transform Step",
section 3):

| Notes file | Split | Time | Peak memory |
| ---------- | ----- | ---- | ----------- |
| `2024q1`, 619 MB zipped, the largest file | Stream | 45.6 s | 463 MB |
| `2024q1` | Buckets of 256 MB, 11 passes | 97.3 s | 714 MB |
| `2024q1` | Stream that stops on the last row, then buckets | About 2.4 minutes | 714 MB |
| `2012q4`, not sorted | Stream stops at row 2, then buckets | 82.4 s | 518 MB |

Not measured: the reads from GCS and the standardize filing step. The findings estimate about one
minute more for the 6.8 GB that 11 passes read through a bucket mount at 100 MB/s.

The choice binds the transform job, the silver writer, and the `silver_files` row of a
`notes_file` unit.

## Decision

1. **One task transforms one whole Notes file.** The transform does not split a Notes file across
   tasks.
2. **Each task logs `unit_slow` when a unit runs past 15 minutes,** as a `warn` event with the
   unit key and the time so far. The task continues.
3. **A `unit_slow` event of a `notes_file` unit reopens this decision.** So does a Notes file
   larger than 1.2 GB zipped, twice the size of `2024q1`.

## Amendment 1: Tasks Start in No Fixed Order

| Field | Value |
| ----- | ----- |
| Date | 2026-11-21 |
| Status | Accepted by the project owner on 2026-11-21 ([STA-463 comment `84a54644`](https://linear.app/state-machine/issue/STA-463#comment-84a54644)) |

**Reason.** "Consequences" says that the job starts the largest files first. The plan gives the
largest files the lowest task indexes, so that sentence assumes that Cloud Run starts tasks in
index order. The Cloud Run documentation says: "If you set --parallelism to greater than 1, tasks
might not follow the index order." The backfill transform runs 10 tasks in parallel.

**Decision.** The plan keeps the largest files at the lowest task indexes, as a hint. The job does
not rely on the order.

**Consequences.**

- A large file can start in the last round. The job then takes at most the time of one Notes file
  longer: about 2.4 minutes for `2024q1` on the measured worst case, against a run of hours.
- The lease, its expiry, and the one task per unit key do not depend on the order.
- The decision of this ADR, one task per Notes file, stays.
- The sentence "The job starts the largest files first" in "Consequences" holds only as a hint.

**Source.** [STA-550 comment `eb197624`](https://linear.app/state-machine/issue/STA-550#comment-eb197624),
2026-11-21: the quote of the Cloud Run documentation (run/docs/container-contract).

## Options Considered

Compared on 2026-11-16, on the measurements in "Context".

| Option | Worst time of `2024q1` against the target | Code | `silver_files` row |
| ------ | ----------------------------------------- | ---- | ------------------ |
| **A (chosen). One task per Notes file** | About 2.4 minutes of 15, before GCS reads and standardize filing | None | Written by the one task |
| B. Every Notes file split into T tasks by hash bucket of `adsh`, then a finalize step | Below A | A part manifest per task and a finalize step | Written by the finalize step |
| C. Split only a file above a size limit, as in B | As A or B | As B | As B |

- A meets the target with a margin of about 6 times on the measured worst case.
- B and C add a step, a manifest, and a barrier for a case that no measured file reaches.

## Consequences

- The backfill transform runs one task per Notes file, about 80 tasks.
- The time of the largest file bounds the job. The job starts the largest files first.
- A file that breaks the target shows as `unit_slow` before it reaches the task timeout of one hour.
- If the decision reopens, option B is the fallback. Its digest needs no change, because the
  silver contract sorts the filing hashes of `content_digest` by filing ID, whatever the parts.

## Open Questions

- The time with GCS reads and the standardize filing step. The first run of the backfill
  transform measures it, and `unit_slow` reports a breach.

## Sources

- Linear document "STA-424 Findings: Silver Parquet Layer and Polars for the Transform Step",
  2026-11-08, section 3.
- [STA-463](https://linear.app/state-machine/issue/STA-463/design-silver-writer-and-transform-job-plan),
  Definition of Done item "Time per task", read on 2026-11-16.
