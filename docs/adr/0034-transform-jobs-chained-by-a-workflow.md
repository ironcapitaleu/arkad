# ADR 0034: The Transform Runs as Cloud Run Jobs That a Workflow Chains

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project owner confirms |
| Date | 2026-11-16 |
| Deciders | The project owner |
| Ticket | [STA-463](https://linear.app/state-machine/issue/STA-463/design-silver-writer-and-transform-job-plan) |

## Context

The data track has four steps after extract: transform, standardize series, checks, and load. Each
step reads what the step before it wrote. The series step must run after the transform units of its
day. Load must not read a unit that the checks did not pass.

Facts that bound the choice:

- Extract runs as two binaries, `daily_filings` once a day and `backfill` by hand
  (`sec/src/bin/` on `main` at commit `fa72cb3`).
- The transform of the largest Notes file, `2024q1`, took 46 s with the streamed split and 97 s
  with the bucket split, on one thread. A `filing_day` unit of 167 filings took 4.6 s (Linear
  document "STA-424 Findings: Silver Parquet Layer and Polars for the Transform Step", section 3
  and section 4).
- STA-463 sets a task timeout of one hour in its Definition of Done.
- The backfill transform has about 80 units. A first series run has 64 entity buckets.
- A run of a job kind needs a plan before its tasks start: the units, the task count, and, for the
  series step, the start time of the run, which every task writes as `computed_at`.
- The write log and the queue port are designs in progress, not code. The design of the write log
  asks this decision whether transform runs as a job or as a service with push delivery.
- The queue port delivers one message per stored document: a `NotesArchiveRef` per Notes file, or
  a `FilingDocumentsRef` per filing. A silver unit holds one Notes file or one filed date, about
  90 filings on a business day.

The choice binds the transform, series, and checks binaries, the deployment of MVP Operations, and
the queue port, which holds the work of the transform.

Terms in this ADR:

- **Barrier:** a step starts only after the execution of the step before it ended.
- **Lease:** a GCS object in a control bucket that lets one run of a job kind write.
- **Plan step:** an execution of one task. It takes the lease of its job kind. It writes the plan
  of the run as a write-once object `plans/<job kind>/<plan id>.json` in the control bucket.
- **Worker step:** an execution with the task count of the plan. Each task reads the plan.

## Decision

1. **Transform, series, and checks run as Cloud Run jobs, not as a service.** Each run of the
   transform and of the series step is a plan step, then a worker step. The checks write no silver
   unit, so they run as one step.
2. **A Cloud Workflows workflow chains the steps.** It starts each execution after the execution
   before it ended. It reads the plan object and starts the worker step with the task count and the
   plan ID of the plan.
3. **The queue holds the work, and the workflow starts the run.** The transform plan step pulls
   the pending messages of its path and groups them by unit key: the Notes period, or the filed
   date of the filing. It writes the plan, then acknowledges the messages. Until the queue port
   exists, the plan step takes the units from the raw store.
4. **Two workflows exist.** `silver-daily` runs after `daily_filings`: transform of the days, the
   series step, the checks, and load. `silver-backfill` runs by hand: transform of the Notes files,
   the series step, the checks, and load.
5. **A failed transform step does not stop the series step.** The series step reads only units
   with a `silver_files` row, and the next series run covers each unit written later. The workflow
   ends as failed.
6. **Load starts only after the checks step succeeded.**
7. **A busy lease stops the workflow** at the plan step, before any write. The messages stay in
   the queue for the next run.

## Options Considered

Compared on 2026-11-16, on the facts in "Context".

| Option | Barrier | Parallel tasks for 80 Notes files | New runtime part |
| ------ | ------- | --------------------------------- | ---------------- |
| **A (chosen). Jobs, a workflow chains plan and worker steps** | End of each execution | Yes | One workflow per path |
| B. Cloud Scheduler starts each job at a fixed time, and each job checks `silver_files` | A guess of the time | Yes | None |
| C. One execution runs every step in one task | In the code | No | None |
| D. A Cloud Run service, started per message by push delivery | None between transform and series | Yes | A service, a topic, and the queue port |
| E. A or D, with an Eventarc trigger that starts the workflow per message | End of each execution | Yes | A trigger. About 90 starts a business day, and each start meets a busy lease while a run is in progress |

- A gives a barrier from the end of an execution, which Cloud Run reports.
- B starts the series step at a fixed time. A slow transform run then misses the series run of
  its day.
- C cannot run 80 Notes files in parallel, and a first series run in one task holds every entity.
- D writes one filing per call, so it gives about 90 small units a day, or it must wait for a
  day to be complete, which no message marks. The series step still runs as a batch over many
  units, so it needs a barrier too.
- E starts many runs that stop at once, and a message that arrives during a run waits for the
  next start.

## Consequences

- MVP Operations deploys two workflows next to the jobs. The workflows hold no business rule.
  Each rule sits in a Rust plan step, which tests cover.
- A run costs one more execution per job kind, for the plan step.
- A manual rerun of one step starts its workflow, or the plan step and the worker step of that
  job kind in order.
- The series step of `silver-daily` runs even after a failed transform step. Its delta then lacks
  the failed units until a later run.

## Open Questions

- If the daily runs must start sooner than after `daily_filings`, a schedule of several runs a day
  can start `silver-daily`. The plan step takes whatever the queue holds, so the steps stay.
- The startup time of one execution is not measured. If it is long, a run with few units can merge
  the plan step into the worker step.

## Sources

- Linear document "STA-424 Findings: Silver Parquet Layer and Polars for the Transform Step",
  2026-11-08, sections 3 and 4.
- [STA-506](https://linear.app/state-machine/issue/STA-506), "Starting Point", read on
  2026-11-16: the question of a job or a service.
- [STA-463](https://linear.app/state-machine/issue/STA-463/design-silver-writer-and-transform-job-plan),
  Definition of Done items "The parallel plan" and "Time per task", read on 2026-11-16.
- `sec/src/bin/` on `main` at commit `fa72cb3`: the binaries `daily_filings` and `backfill`.
