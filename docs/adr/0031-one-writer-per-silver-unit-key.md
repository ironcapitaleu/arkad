# ADR 0031: One Writer per Silver Unit Key, Through a Lease per Job Kind

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project owner confirms |
| Date | 2026-11-16 |
| Deciders | The project owner |
| Ticket | [STA-463](https://linear.app/state-machine/issue/STA-463/design-silver-writer-and-transform-job-plan) |

## Context

Silver writes units. A unit key is the dataset, the source, the unit kind, and `unit_key`. A unit
becomes visible when its `silver_files` row exists. The current unit of a unit key is the unit with
the latest `written_at`, a `timestamp[ms, UTC]`. Silver keeps every older unit.

Two writers of one unit key at the same time can give two units with an equal `written_at`. Then
the current unit is not defined, and two readers can pick different units. The review of
[ironcapitaleu/arkad#441](https://github.com/ironcapitaleu/arkad/pull/441) (review of commit
`31f7aaa`) named the case: a daily run and a replay write `filing_day=2026-11-06` in the same
millisecond, with different `transform_version` values.

Two other facts bound the choice:

- A `series_base` run and a `series_delta` run must never overlap. Both write new unit keys for
  each run, so a rule per unit key cannot stop the overlap. A rule per run is needed.
- Assumption, not checked in this session: Cloud Run starts a new attempt of a task only after the
  previous attempt of that task ended. The network policy of the session blocked the Cloud Run
  documentation on 2026-11-16.

The choice binds the silver writer, the silver reader, the transform jobs, the series job, and the
replay jobs of the as-is and the standardized layer.

Terms in this ADR:

- **Job kind:** the jobs that can write one unit key. There are three: `transform-notes` writes the
  `notes_file` units, `transform-daily` writes the `filing_day` units, and `series` writes the
  `series_base` and `series_delta` units. A replay runs as its job kind.
- **Lease:** a GCS object that names the one run of a job kind that can write.

## Decision

1. **One run of a job kind writes at a time.** The plan step of a run creates the lease object
   `_control/leases/<job kind>` with `ifGenerationMatch=0`. The lease holds the plan ID and an
   expiry time. If the object exists and its expiry has passed, the plan step replaces it with
   `ifGenerationMatch=<generation>`. Otherwise the run stops before any write.
2. **Each task checks the lease before its first write.** If the lease does not name the plan of
   the task, or its expiry has passed, the task writes nothing and fails.
3. **The orchestrator deletes the lease after the worker execution ends,** with
   `ifGenerationMatch=<generation>`. It deletes the lease after a failed execution too.
4. **Inside one run, one task writes one unit key.** The plan lists the units, and task `i` takes
   the units at the positions `i`, `i + n`, and so on, for `n` tasks.
5. **`written_at` rises with each unit of a key.** The writer reads the current unit of the key
   for rule 2 of a write. It sets `written_at` to the later of the time now and the `written_at` of
   that unit plus 1 ms. So clock skew between two machines cannot give a tie.

The rule "the current unit is the unit with the latest `written_at`" stays as it is.

## Options Considered

Checked on 2026-11-16 against the silver contract on `main` at commit `fa72cb3`.

| Option | Ties on `written_at` | Base and delta runs overlap | Change to the read rule |
| ------ | -------------------- | --------------------------- | ----------------------- |
| **A (chosen). A lease per job kind, one task per unit key, rising `written_at`** | None | Never | None |
| B. Concurrent writers, and a tie-break by the greater `files` value | Defined by the tie-break | Possible. A run lease is still needed | Every reader adds the tie-break |
| C. A lease per unit key, held by each task until the task timeout | None | Possible. A run lease is still needed | None |

- A covers both cases with one mechanism, and readers keep the rule they have.
- B lets two runs do the same work, and each reader must implement the tie-break the same way.
- C needs one lease object per unit key, about 80 for a backfill, and still a lease per series run.

## Consequences

- A second run of a job kind fails at its plan step while the first run holds the lease. A daily
  run that meets a replay in progress fails, and the next daily run covers its dates.
- A crashed orchestrator leaves a lease until its expiry. The expiry is the plan time plus the
  longest time a run of the job kind can take. That time is the rounds of tasks, times the task
  timeout, times the attempts per task.
- A job execution started by hand, outside the orchestrator, has no plan of its own. Its tasks
  fail the lease check and write nothing.
- Different job kinds can run at the same time. They write different unit keys. A series run that
  meets a transform run reads only the units with a `silver_files` row, and the next series run
  covers the units written during the run.

## Open Questions

- If a later design lets two job kinds write one unit key, this decision opens again.
- If silver moves to a table format with its own commit, such as Delta Lake, the lease can go.
- If two attempts of one task can run at the same time, decision 4 does not hold. The feature of
  the lease checks the assumption of "Context" against the Cloud Run documentation.

## Sources

- The review of commit `31f7aaa` on
  [ironcapitaleu/arkad#441](https://github.com/ironcapitaleu/arkad/pull/441), finding 🟣 1, and
  the reply that left the case to the transform design.
- [STA-425](https://linear.app/state-machine/issue/STA-425): the silver contract, with the rule of
  the current unit and the type of `written_at`, read on 2026-11-16.
- [STA-463](https://linear.app/state-machine/issue/STA-463/design-silver-writer-and-transform-job-plan),
  Definition of Done item "One writer per unit key", read on 2026-11-16.
