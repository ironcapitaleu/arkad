# Silver Writer and Transform Jobs

> **Status:** proposed, for review by the owner. [ADR 0031](../../../docs/adr/0031-one-writer-per-silver-unit-key.md),
> [ADR 0032](../../../docs/adr/0032-byte-form-of-content-digest.md),
> [ADR 0033](../../../docs/adr/0033-transform-jobs-chained-by-a-workflow.md), and
> [ADR 0034](../../../docs/adr/0034-one-task-per-notes-file.md) hold the decisions of this document.
> All four are "Proposed".
>
> **Marks:** "Measured" means checked on 2026-11-08 against a downloaded SEC file, as the silver
> prototype findings record it. "Estimate" means not measured. Every other statement is a proposal.

This document sets how the code meets the silver contract of [`silver_layer.md`](silver_layer.md).
It covers how a unit reaches silver, how the series step runs, and how the jobs run and fail. It does not
change the contract. The per-filing work of the transform, its states, parsers, standardize filing,
and checks, belongs to the per-filing transform design. This document calls that work "the filing
step".

## Reading Guide

- [Overview](#overview) gives the diagram of the jobs. Read it first.
- [The Silver Writer](#the-silver-writer) and [Byte Form of `content_digest`](#byte-form-of-content_digest)
  describe the code that every writer shares.
- [The Series Step](#the-series-step) and [Series Runs and Deltas](#series-runs-and-deltas) describe
  the standardize series super-state.
- [One Writer per Unit Key](#one-writer-per-unit-key), [The Parallel Plan](#the-parallel-plan), and
  [Time per Task](#time-per-task) describe the jobs.
- [Failure Handling](#failure-handling) lists what each failure does to a unit and a run.
- [Implementation Tickets](#implementation-tickets) lists the features in their order.
- [Open Points](#open-points) lists what this design leaves open.

## Overview

```mermaid
flowchart TB
    classDef step fill:#dae8fc,stroke:#6c8ebf
    classDef store fill:#e6e6e6,stroke:#666666
    classDef ctl fill:#fff2cc,stroke:#bf9000

    subgraph W["Workflow silver-daily or silver-backfill"]
        direction TB
        TP["transform plan<br/>1 task"]:::step
        TW["transform work<br/>1 task per unit"]:::step
        SP["series plan<br/>1 task"]:::step
        SW["series work<br/>1 task per bucket set"]:::step
        CK["checks"]:::step
        LD["load"]:::step
        TP -- "barrier" --> TW -- "barrier" --> SP -- "barrier" --> SW -- "barrier" --> CK -- "barrier, passed only" --> LD
    end

    subgraph C["Control bucket"]
        LT[("lease<br/>transform-notes or<br/>transform-daily")]:::ctl
        LS[("lease<br/>series")]:::ctl
        PL[("plans")]:::ctl
    end

    subgraph S["Silver bucket"]
        DS[("datasets")]:::store
        SF[("silver_files")]:::store
    end

    TP -- "take" --> LT
    TP -- "write" --> PL
    TW -- "check" --> LT
    TW -- "units" --> DS
    TW -- "rows last" --> SF
    SP -- "take" --> LS
    SP -- "read" --> SF
    SP -- "write" --> PL
    SW -- "check" --> LS
    SW -- "base or delta units" --> DS
    SW -- "rows last" --> SF
    CK -- "read" --> SF
    LD -- "read" --> SF
```

- The workflow deletes each lease after the worker step of its job kind ends. The diagram leaves
  these edges out.
- `silver-daily` runs after the extract job `daily_filings`, with the kind `transform-daily`.
  `silver-backfill` runs by hand, with the kind `transform-notes`.
- The checks and load have their own designs. This document fixes only their place in the chain.

## The Silver Writer

### Where the Code Lives

| Part | Crate | Holds |
| ---- | ----- | ----- |
| Ports | `storage` | `CreateRepository`, `ReadRangeRepository`, and `LeaseStore`, each a standalone trait with a fake and a filesystem backend |
| GCS backend | `storage-gcs`, a new crate | The three ports on the GCS JSON API, and `ReadRepository` and `ListRepository` for the control objects |
| Silver code | `sec`, module `silver` | `SilverRecord`, `DigestRow`, the row types, the writer, the reader, the plans, and the task runner |
| Series rules | `xbrl`, module `core::series` | `standardize_series`, the order of the rules per entity |

The ports are new traits, so no existing trait changes (`AGENTS.md`, "Open-Closed Principle"):

- `WriteRepository::persist` returns `()`. The writer needs the size, the CRC32C, and the
  generation of each object, and a write that fails if the object exists. So `CreateRepository`
  has `create(key, bytes) -> Result<StoredObject, WriteError>`.
- A Parquet reader reads the footer and the row groups by byte range. `ReadRangeRepository` has
  `read_range(key, range) -> Result<Bytes, ReadError>`. The size comes from `silver_files`.
- `LeaseStore` has `acquire` and `current`. The workflow deletes a lease through the GCS API. See [One Writer per Unit Key](#one-writer-per-unit-key).

The GCS backend sets `ifGenerationMatch=0` on each create, and sends the CRC32C of the bytes, so
GCS rejects a damaged upload. It returns the size and the CRC32C from the object resource that GCS
answers. The bucket mount gives neither the precondition nor the CRC32C, so silver does not use it.
`storage-gcs` sits apart from `storage` for the reason of
[ADR 0005](../../../docs/adr/0005-storage-ports-crate.md): only a composition root compiles it. Its
client must use `rustls`, and `cargo tree -i openssl-sys` must find no package.

The filesystem backend opens each file with `create_new` and computes the CRC32C itself. Tests and
local runs use it.

### `SilverRecord` and `DigestRow`

```rust
/// A row type of one silver dataset and its Arrow form.
pub trait SilverRecord: Sized {
    const DATASET: &'static str;
    const SCHEMA_VERSION: u16;
    fn schema() -> SchemaRef;
    fn to_batch(rows: &[Self]) -> Result<RecordBatch, SilverError>;
    fn from_batch(batch: &RecordBatch) -> Result<Vec<Self>, SilverError>;
}

/// A row that enters `content_digest`.
pub trait DigestRow {
    fn write_group_id(&self, out: &mut ByteForm);
    fn write_row_key(&self, out: &mut ByteForm);
    fn write_row(&self, out: &mut ByteForm);
}
```

- One row type per dataset: `AsIsFactRow`, `FilingRow`, `CandidateRow`, `StandardizedFactRow`,
  `FindingRow`, `TaxonomyElementRow`, and `SilverFileRow`. Each wraps the `xbrl` type and adds the
  provenance and run columns of the contract.
- `to_batch` builds the Arrow arrays column by column. It does not go through JSON. The prototype
  found that the `arbitrary_precision` feature of `serde_json`, which `xbrl` turns on, breaks the
  Arrow JSON decoder.
- `from_batch` takes the `schema_version` of the unit from its `silver_files` row. It rejects a
  batch whose fields differ from the schema of that version. Each row type keeps the schema of
  every version that a stored unit can hold, because a unit stays current until a replay.
- `DigestRow` is a separate trait. A type can have a digest without an Arrow form, for example the
  fixture rows of the golden test.

### Schema Tests

Each dataset has three tests:

1. **Round trip.** Rows to a batch and back give equal rows. The fixture covers a numeric value at
   38 digits, text with non-ASCII characters, `Infinite` decimals, and a row with every nullable
   field empty.
2. **Snapshot.** The schema prints as text, one line per field: name, type, and nullability. The
   printer belongs to `sec::silver`, not to the `Display` of Arrow, which changes between releases.
   The text is checked in. A change of the text needs a new `SCHEMA_VERSION`.
3. **Folder labels.** No field name equals a folder label of the path of the dataset: `source` and
   `unit_key`, or `execution` and `task` for `silver_files`.

### Rules of a Write

The writer takes the unit key, the inputs, the row stream, and the run context. It follows the four
rules of the contract in this form:

1. **Digest.** It computes `input_digest` from the sorted list of input keys and SHA-256 values.
2. **Skip.** It reads the current unit of the key through the silver reader. If that unit has the
   same `input_digest` and `transform_version`, it writes nothing and reports the unit as skipped.
3. **Parts.** It writes each part to
   `<dataset>/source=<source>/unit_key=<key>/<digest16>-v<version>/<execution_id>-<attempt>/part-<n>.parquet`
   through `CreateRepository`. The series datasets leave out `source=`. Each part:
   - sorts its rows by `filing_id`, then by the value key, or by `entity_bucket`, then
     `filing_id` for `candidates`.
   - has row groups of about 100,000 rows. `candidates` starts a new row group at each change of
     `entity_bucket`.
   - uses zstd at level 3.
   - holds the footer entries `dataset`, `schema_version`, `unit_key`, and `transform_version`,
     through `WriterProperties::set_key_value_metadata`.
4. **List row.** It writes the `silver_files` row last, to
   `silver_files/execution=<execution_id>/task=<n>-<attempt>.parquet`, with the size and CRC32C of
   each part as `create` returned them. It sets `written_at` to the later of the time now and the
   `written_at` of the current unit plus 1 ms ([ADR 0031](../../../docs/adr/0031-one-writer-per-silver-unit-key.md)).

A transform task writes one unit key in each of four datasets: `as_is_facts`, `filings`,
`candidates`, and the filing rows of `findings`. A series task writes one unit key in
`standardized_facts` and one in `findings`. Each task writes the `silver_files` rows of its units
in its one list file, at its end. So the units of one task appear together, or not at all.

The writer never holds a whole unit. It flushes a part when the rows of the part reach about 256
MB of input text, and drops them. For each digest group, it keeps only the 32-byte hash.

### The Silver Reader

One reader serves every Rust caller. It reads `silver_files` and never lists a dataset folder.

- `current_units(dataset, source)` takes, per unit key, the unit with the latest `written_at`.
- `current_units_as_of(dataset, source, t)` takes the units with `written_at` at or before `t`.
- `series_units(dataset)` takes the newest `series_base` of a run that counts, and every
  `series_delta` of a run that counts after it, as "Series Deltas" of the contract says.
- `read_rows::<R>(unit, filter)` reads the parts through `ReadRangeRepository`, prunes row groups
  by their statistics, and returns rows of type `R`.
- The current facts rule of the contract, one unit per filing, runs on top of these calls.

## Byte Form of `content_digest`

[ADR 0032](../../../docs/adr/0032-byte-form-of-content-digest.md) fixes the form. In the code:

- `ByteForm` is a byte buffer with one method per Arrow type of the contract, for example
  `decimal(&BigDecimal)` and `date(NaiveDate)`, plus `null()`, `list(len)`, and `struct_(fields)`.
- Each `DigestRow` writes its columns in schema order and skips `execution_id`, `loaded_at`, and
  `computed_at`.
- The writer hashes one digest group when the group is whole. The filing step hands over one whole
  filing, and the series step one whole entity. So the writer sorts the rows of one group, hashes
  them, and keeps the group ID and the hash. At the end of the unit, it sorts the pairs and computes
  `content_digest`.
- The golden test holds one fixture unit per dataset and a table of `transform_version` to
  expected digest.

## The Series Step

### Entity Buckets

`entity_bucket` is `xxh3_64` with seed 0 over the UTF-8 bytes of `entity_scheme`, `:`, and
`entity_id`, modulo 64. The split of a Notes file uses the same hash with a fixed seed, so no
second hash enters the code. The count 64 comes from "Cloud Run Sizing" of the contract. A run with
`n` tasks gives task `t` the buckets `b` with `b mod n = t`.

### Per Entity

`xbrl::core::series::standardize_series(candidates, filings, conflicts) -> SeriesOutcome` runs the
rules of one entity in this order:

1. `newest_filing(candidates)` gives the `Series`: step 3 of the dedup order.
2. `derive_q4(&series, filings)` gives the fourth quarters.
3. `derive_from_closed_groups(&series, &fourth_quarters)` gives the roll-up values. Steps 2 and 3
   are step 4 of the dedup order.
4. `added` is the fourth quarters, then the roll-up values.
5. `check_roll_ups(&series, &added)` gives `RollUpMismatch` and `NegativeOther` findings.
6. `check_all_sums(&series, &added)` gives `SumMismatch` findings. Steps 5 and 6 take the same
   `added`.
7. `check_chronology` runs on the series with `added` merged in, with `conflicts` as the findings of
   dedup. It gives `Gap`, `Overlap`, `Restatement`, and `SuspiciousPeriod`, then the conflicts.

`SeriesOutcome` holds the standardized facts of step 7 and the findings of steps 5, 6, and 7.
`standardize_series` returns no error, because none of the functions it calls returns one.

- `candidates` holds one `candidates` unit per filing, by the current facts rule of the contract.
- `filings` holds the `filings` rows of the entity, for the forms and the fiscal year end.
- `conflicts` holds the `Conflict` rows of `findings` with `step = filing` for the filings of
  `candidates`. The series step writes them again with `step = series`.

Segment values pass through step 1 with their dimensions. Steps 2 to 7 read only values without
dimensions, by the rules of each function. The series step never calls `segment::reconcile`, so
silver gets no line of the segment reconciliation
([ADR 0030](../../../docs/adr/0030-segment-reconciliation-per-report.md)). A test runs the series
step on a filing with segment values and asserts that no output row carries a rule that
`ReconciliationLine::role_rule` returns.

### The Super-State

The series step is a super-state per set of buckets, run once per task:

| State | Input | Output |
| ----- | ----- | ------ |
| `ReadSeriesInputs` | The plan, the task index | The candidates, filings, conflicts, and current series rows of the entities of the task |
| `StandardizeSeries` | The inputs per entity | One `SeriesOutcome` per entity |
| `CompareWithCurrent` | The outcomes and the current rows | The rows to write, with `change` and `computed_at` |
| `WriteSeriesUnit` | The rows to write | The `silver_files` row of the unit |

- `ReadSeriesInputs` reads the row groups of the buckets of the task from every `candidates` unit
  of the plan. It reads `filings` and the `Conflict` rows with a column projection, and keeps the
  rows of the entities of the task.
- `StandardizeSeries` runs `standardize_series` on N worker threads, one entity at a time.
- A bucket holds about 0.9 million candidate rows (estimate: about 450,000 filings of about 130
  candidate rows each, over 64 buckets). At about 300 bytes per row in memory, a bucket takes about
  270 MB, within the 2 GiB of a series task.

## Series Runs and Deltas

### The Series Plan

The plan step of a series run takes the lease `series`, reads `silver_files`, and writes the plan:

| Field | Meaning |
| ----- | ------- |
| `plan_id`, `computed_at` | The ID of the run, and its start time. Every task writes this `computed_at` |
| `mode` | `base` or `delta` |
| `run_tasks` | The task count. A delta run has `ceil(entities / 100)` tasks, at most 64. A base run has 64 |
| `transform_version` | The version of the binary |
| `inputs` | The current `candidates` and `filings` units, and the units of the current series rows, by their `files` |
| `entities` | The entities to compute, with their bucket |

Every task reads the plan, not `silver_files`. So every task of a run reads the same units, even if
a transform run writes during the series run.

### Mode

The plan step picks `base` if one of these holds. Otherwise it picks `delta`.

- No run that counts wrote a `series_base`.
- The newest `series_base` of a run that counts has another `transform_version` than the binary.
- The entity set of a delta covers more than half of all entities with a current `candidates`
  unit. A delta that rewrites most rows costs as much as a base, and the next read then starts from
  the new base.
- The operator sets `ARKAD_SERIES_MODE=base`.

A base run computes every entity with a current `candidates` unit. It writes each row with
`change = new`, and each open finding again with `change = new`. The first base has the unit
keys `base-t<n>`. A later base has `base-<computed_at>-t<n>`, as the contract gives them.

### Entity Set of a Delta

The plan step compares two sets of `candidates` units:

- **Now:** the current units, by the current facts rule.
- **Then:** the `candidates` units in the `inputs` field of the plan of the latest run that counts.
  The plan ID of a series run is its `computed_at`, so the plan step finds that plan from the
  `computed_at` of the run in `silver_files`. The control bucket keeps every plan.

The entity set is the entities of each unit in "now" and not in "then", plus the entities of each
unit in "then" and not in "now". The second part covers a unit that a replay replaced, so an entity
that the replay drops gets `removed` rows. The plan step reads the entities of a unit from its
`entity_scheme` and `entity_id` columns only.

Each unit with a `written_at` after the `computed_at` of the latest run that counts is in "now" and
not in "then". So the set holds every unit that the contract asks for. It also holds a unit whose
`written_at` lies before that `computed_at` but whose `silver_files` row appeared after the plan
of that run read the list. A rule by time alone misses such a unit.

### Compare

`CompareWithCurrent` compares per key of the contract, for each entity of the task:

| Current row | New row | Written |
| ----------- | ------- | ------- |
| None, or `removed` | Present | The new row, `change = new` |
| Present | Present, other byte form | The new row, `change = changed` |
| Present | Present, same byte form | Nothing |
| Present | None | A copy of the current row, `change = removed` |

- Two rows have the same byte form if their bytes of [ADR 0032](../../../docs/adr/0032-byte-form-of-content-digest.md)
  are equal without the run columns, `computed_at`, and `change`.
- For `findings`, a finding is current while it is open. A new finding gives `new`. A new `reason`,
  `confirmed_by`, or `confidence` gives `changed`. An open finding that the run does not give again
  gives a copy with `change = resolved`.
- Every written row gets the `computed_at` of the plan.
- Each task writes its `series_delta` unit with the key `<computed_at>-t<n>`, also with no changed
  row. `<computed_at>` prints as `YYYY-MM-DDTHHMMSS.mmmZ`, so two runs in one minute get two keys.
- The run counts when `silver_files` holds `run_tasks` distinct unit keys of its `execution_id`.
  Until then, no read takes its units, and the next plan compares against the run before it.

## One Writer per Unit Key

[ADR 0031](../../../docs/adr/0031-one-writer-per-silver-unit-key.md) fixes the rule. The parts in
the code:

- `LeaseStore::acquire(kind, plan_id, expires_at)` creates `leases/<kind>` in the control bucket
  if absent. It replaces the object if its expiry passed. Otherwise it returns `LeaseHeld` with
  the plan that holds it.
- The plan step computes `expires_at`: the plan time, plus `ceil(units / parallel tasks)` times
  the task timeout times the attempts per task, plus 10 minutes for the start of the worker step.
  For 80 Notes files, 10 parallel tasks, a timeout of 1 hour, and 2 attempts, that is 16 hours and
  10 minutes.
- Each worker task calls `LeaseStore::current(kind)` before its first write. It fails with
  `LeaseLost` if the lease names another plan, or if less than one task timeout remains before
  the expiry. Cloud Run stops an attempt at its timeout, so the task ends before the expiry.
- The workflow deletes the lease after the worker step, with the generation that `acquire` returned.

The control bucket holds the leases and the plans. It is not the silver bucket, so the cleanup of
unread silver files never meets a control object.

## The Parallel Plan

### Jobs and Steps

[ADR 0033](../../../docs/adr/0033-transform-jobs-chained-by-a-workflow.md) fixes the chain. Three
binaries run the steps, each in its own image ([ADR 0002](../../../docs/adr/0002-static-binaries.md)):

| Binary | Step | Plan | Tasks of the worker step |
| ------ | ---- | ---- | ------------------------ |
| `transform` | `ARKAD_JOB_STEP=plan` | Lists the units: each Notes period with a stored file, or each filed date from `ARKAD_FROM` to `ARKAD_TO` | One per unit |
| `transform` | `ARKAD_JOB_STEP=work` | | Task `i` takes unit `i` of the plan |
| `series` | `ARKAD_JOB_STEP=plan` | See [The Series Plan](#the-series-plan) | `run_tasks` |
| `series` | `ARKAD_JOB_STEP=work` | | Task `t` takes its buckets |
| `checks` | One step. It writes no silver unit, so it takes no lease | | Set by the design of the checks |

- The plan of `transform-notes` sorts the Notes files by size, largest first, so the largest files
  get the lowest task indexes. This assumes that Cloud Run starts tasks in index order. The
  feature of the plans checks the assumption.
- The plan of `transform-daily` takes the window of `daily_filings`, the last four days. A day
  whose inputs did not change skips by rule 2 of a write.
- The worker step reads `ARKAD_PLAN`, the plan ID that the workflow passes.

### Threads Inside a Transform Task

```mermaid
flowchart LR
    classDef th fill:#dae8fc,stroke:#6c8ebf
    R1["reader<br/>num.tsv"]:::th --> J["join by adsh"]:::th
    R2["reader<br/>txt.tsv"]:::th --> J
    J -- "one filing,<br/>bounded channel" --> F1["filing worker 1"]:::th
    J --> F2["filing worker N"]:::th
    F1 -- "rows of one filing" --> WR["writer"]:::th
    F2 --> WR
```

- **One reader per table stream.** Each reader inflates one table of the ZIP. The join hands one
  whole filing to the channel. A `filing_day` unit has one reader, which reads the instances of the
  day one after the other.
- **N filing workers.** Each worker runs the filing step on one filing: parse, standardize filing,
  and the per-filing checks. N is the number of cores of the task, at least 1. Measured on one
  thread, `2024q1` took 45.6 s, so N bounds cost, not the target.
- **One writer.** It receives the rows of whole filings and writes the parts of every dataset of
  the unit. It hashes each filing when it arrives.
- **Bounded channels** of 2 × N filings cap the memory. The largest filing has 27,529 rows in
  `num.tsv` of `2025q3` (measured).
- In the bucket split, the reader sends an end-of-pass message with the count of filings of the
  pass. The writer flushes the part of the bucket when it received that many filings.

The filings arrive at the writer in any order. So the content of each part can differ between two
runs. The rows of each part are sorted, and `content_digest` does not depend on the parts.

### Barriers

| Before | After | Barrier |
| ------ | ----- | ------- |
| Extract `daily_filings` | Transform plan | End of the extract execution |
| Transform work | Series plan | End of the transform work execution, also if it failed |
| Series work | Checks | End of the series work execution |
| Checks | Load | The checks execution succeeded |

A `series_base` run and a `series_delta` run never overlap, because both take the lease `series`.

## Time per Task

[ADR 0034](../../../docs/adr/0034-one-task-per-notes-file.md) keeps one task per Notes file. The
target is 15 minutes per largest Notes file, within the task timeout of one hour. The
prototype measured about 2.4 minutes for the worst case of `2024q1`, on one thread.

- Each task logs `unit_slow` when one unit runs past 15 minutes.
- If the decision reopens, the fallback splits a Notes file into T tasks by hash bucket of `adsh`.
  Each task writes its parts and a part manifest to the control bucket: its `files`, its counts,
  and the group IDs and hashes of its filings. A finalize step, after a barrier, checks that all T
  manifests exist. It merges the pairs, sums the counts, and writes the one `silver_files` row of
  the unit.

| Job | Task timeout | Memory | Parallel tasks |
| --- | ------------ | ------ | -------------- |
| `transform`, Notes | 1 hour | 2 GiB. Measured peak 714 MB with the bucket split | 10 |
| `transform`, days | 1 hour | 1 GiB | 4 |
| `series`, work | 30 minutes | 2 GiB | 16 |
| Plan steps | 10 minutes | 512 MiB | 1 |

## Failure Handling

| Failure | Example | Unit | Task and run | Retry |
| ------- | ------- | ---- | ------------ | ----- |
| Transient | GCS answers 429 or 503, a read times out, a connection drops | Fails if three attempts fail | The task exits with an error | Each request: 3 attempts, waits of 1 s and 2 s with jitter, as for SEC requests. Then Cloud Run retries the task once. A later run skips each written unit by rule 2 |
| Bad filing | The parser rejects one filing | Written without the filing. `dropped_rows` counts it as `rejected_filing` | The run summary counts it | None |
| Bad unit | The SHA-256 of the raw file differs from its metadata, the ZIP is damaged, a table is missing, too many filings rejected | Fails. Its attempt files stay unread | The task exits with an error. The run fails | Cloud Run retries the task once, and the unit fails again |
| Timeout | A unit runs past one hour | Fails. Its attempt files stay unread | Cloud Run stops the task | Cloud Run retries the task once. A second timeout fails the execution. `unit_slow` warns at 15 minutes |
| Out of memory | A task passes its memory limit | As a timeout | As a timeout | As a timeout |
| Bug, with a panic | `panic = "abort"` of the release profile ends the process | As a timeout | As a timeout. A series run with a panic never counts | As a timeout. Each later run fails the same way until a fix |
| Bug, wrong output | A rule gives a wrong value and no error | Written | Not detected by the run | A fix gets a new `transform_version`. A replay of the as-is layer writes new units, and the next series run writes a base |
| Bad silver unit in the series | A `candidates` unit cannot be read or has an unknown `schema_version` | The series unit of the task is not written | The task fails. The run never counts, and each later run fails the same way | A replay of the transform unit writes a new current unit. The next series run then reads it |
| Lease held | Another run of the job kind is in progress | Not started | The plan step fails before any write. The workflow stops | The next scheduled run |
| Lease lost | A worker task finds another plan in the lease, or less than one task timeout before its expiry | Not started | The task fails before any write | Cloud Run retries the task once, and it fails again |

- The per-filing transform design decides which filing the parser rejects, and the share of
  rejected filings that fails a unit.
- A series task that fails leaves its run without a count. The next series run compares against the
  run before it, so it recomputes every entity of the failed run.
- The design of failure modes and dead letters decides where a failed unit goes after the retry,
  and which signal raises an alert.

### Run Summary

Each task logs one JSON event `task_summary` at its end, with these keys in `context`:

| Key | Meaning |
| --- | ------- |
| `job_kind`, `plan_id`, `task_index`, `attempt` | The task |
| `units_planned`, `units_written`, `units_skipped`, `units_failed` | Counts |
| `failed_units` | The unit key and the error kind of each failed unit: `transient`, `bad_unit`, or `lease` |
| `rejected_filings` | Count over the units of the task |
| `rows_written` | Per dataset |
| `slowest_unit`, `slowest_unit_seconds` | The unit with the longest time |
| `series` | For a series task: `mode`, `entities`, rows `new`, `changed`, `removed`, findings `new`, `changed`, `resolved` |

The task also logs `unit_written`, `unit_skipped`, `unit_failed`, and `unit_slow` for each unit.
The summary of a run is the set of `task_summary` events with its `plan_id`. A task exits with an
error if a unit failed. The workflow ends as failed if a step failed.

## Implementation Tickets

Each ticket fits one PR under 400 reviewable lines (estimate). Proposed. The list gives the order.

1. FEATURE: the ports `CreateRepository` and `ReadRangeRepository` in `storage`, with fakes and
   the filesystem backend.
2. FEATURE: the port `LeaseStore` in `storage`, with a fake and the filesystem backend.
3. FEATURE: the crate `storage-gcs` with the GCS backend of the three ports.
4. FEATURE: `ByteForm`, `DigestRow`, and `content_digest`, with the golden test.
5. FEATURE: `SilverRecord` for `as_is_facts` and `filings`, with the schema tests.
6. FEATURE: `SilverRecord` for `candidates` and `silver_files`.
7. FEATURE: `SilverRecord` for `standardized_facts`, `findings`, and `taxonomy_elements`.
8. FEATURE: the silver writer: rules of a write, parts, footer, and `written_at`.
9. FEATURE: the silver reader: current units, units as of a time, series units, and every
   `schema_version`.
10. FEATURE: the plan objects of the transform and series jobs, and the lease check of a task.
11. FEATURE: the task runner of the transform: threads and one unit per task.
12. FEATURE: failure handling and the run summary of the transform task.
13. FEATURE: `xbrl::core::series::standardize_series`.
14. FEATURE: the series plan: mode, entity set, and inputs.
15. FEATURE: `compare_series`: the rows of a delta, with `removed` and `resolved` copies.
16. FEATURE: the standardize series super-state per set of entity buckets, and the binary `series`.
17. FEATURE: the workflows `silver-daily` and `silver-backfill`.

## Open Points

- **Gap in the contract: the `Conflict` rows of an entity.** The series step copies the `Conflict`
  findings of each entity. `findings` sorts its filing rows by `filing_id`, with no entity bucket.
  So each task of a base run reads every `Conflict` row of silver. The prototype found 99
  `Conflict` rows in one 10-Q. The cost is not measured. The owner decides whether `findings` gets
  `entity_bucket` as `candidates` has it.
- **`MixedStarts`.** The contract lists it as a kind of `findings`. No function of `xbrl::core`
  returns it. The series step writes the findings that its functions return, so silver holds no
  `MixedStarts` row until `xbrl` has a function that returns one.
- **Cost of the byte form.** Not measured. The feature of the digest measures it on `2024q1`.
- **Time of a series task.** Not measured. The feature of the super-state measures one bucket of a
  base run.
- **The startup of an execution.** Not measured. ADR 0033 names it.
- **A signal for a rule change.** The design of the replay of the standardized layer adds a signal
  that starts a base run. The plan step reads it next to the conditions of [Mode](#mode).
- **Gold.** The load step and its ledger belong to the gold design.
