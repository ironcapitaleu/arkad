# ADR 0003: The Backfill Runs the Notes Step and the Catch-Up at the Same Time

| Field | Value |
| ----- | ----- |
| Status | Accepted by the owner on 2026-11-13. Built in ironcapitaleu/arkad#513, with the gap check of ironcapitaleu/arkad#517. ironcapitaleu/arkad#519, still open, adds the `HEAD` condition of rule 3 |
| Date | 2026-11-13 |
| Deciders | The project owner |
| Ticket | [STA-435](https://linear.app/state-machine/issue/STA-435). The gap check is [STA-511](https://linear.app/state-machine/issue/STA-511) |

## Context

A first load of the store has two steps. The Notes step downloads one Notes file per period, about
80 files and 28 GB. The catch-up reads each daily index from the cut-over date to yesterday, and
the filings and `submissions` files that it lists. The cut-over date is the last day of the newest
Notes period, so the two paths overlap by one day.

Until ironcapitaleu/arkad#513, the `backfill` binary ran the steps in sequence. It took the
cut-over date from the Notes files that the Notes step had stored. On the owner's machine, the two
steps took about the same time, so a run took the sum of both.

The steps use different resources. The Notes step sends few requests and waits on bandwidth. The
catch-up sends many small requests and waits on the SEC and on the rate limiter. So the owner
expected little slowdown when both run at once.

To start the catch-up before the Notes step ends, the run needs the cut-over date before any
download. The risks of a wrong date are not equal:

- A date that is too early costs only overlap. Every write is idempotent.
- A date that is too late leaves days in neither path. If no step reports it, the gap stays.

## Decision

1. **`backfill().all()` runs the Notes step and the catch-up at the same time**, with one
   `execution_id`. The `backfill` binary calls it.
2. **One `HEAD` request sets the cut-over date, before any download.** The request asks for the
   file of the newest period of the list.
   - If the SEC answers 200, the cut-over date is the last day of that period.
   - If it answers 404, the cut-over date is the last day of the next older period of the list.
   - Any other answer, after the retries on a transient failure, stops the run before any
     download.
   - The date of today plays no part, and the run reads no stored file for the date.
3. **The run checks for a gap after both steps end.** `AllRunSummary::result()` fails, and the
   binary exits with an error, in two cases:
   - A Notes period failed.
   - The `HEAD` request found the newest period, and no stored period ends on or after the
     cut-over date. For example, the download of that file got a 403.
4. **A repeat run fills a gap.** It skips each stored file that passes its size and SHA-256 check,
   so it downloads only what is missing.
5. **The default cap on Notes files in flight stays 2** (`DEFAULT_FILES_IN_FLIGHT`). It bounds the
   memory of the downloads. `ARKAD_FILES_IN_FLIGHT` overrides it.

## Options Considered

| Option | Cut-over date from | Run time | A gap is found |
| ------ | ------------------ | -------- | -------------- |
| Sequence | The stored Notes files, after the Notes step | Sum of both steps | Before the catch-up starts: a failed period stops the run |
| **Parallel, one `HEAD` request** | **The newest period of the list** | **About the longer step** | **After the run, as a failed run** |
| Parallel, one `HEAD` request per month | Every listed month | About the longer step, plus one request per listed month. On 2026-11-13 the list held 13 months, about 1.5 s at the rate limit | Before any download, if a file is missing. A failed download is still found only after the run |
| Parallel, cut-over date from the store | The stored Notes files of an earlier run | About the longer step | Before the run. On an empty store there is no date, so the first load cannot use it |

- A `HEAD` request per month only lets a run fail sooner. It does not prevent a failure, because a
  download can still fail after the request answers 200.
- The store gives a date only after an earlier run stored the Notes files.
  ironcapitaleu/arkad#515 adds `stored_cut_over_date` for `daily_filings`, which has no `HEAD`
  request. `daily_filings` does not call it yet.

## Measurements

Measured on 2026-11-13 in the cloud container (4 cores, about 13 to 15 MB/s to the SEC). Each run
used a fresh store, the Notes periods `2025q1`, `2025q2`, and `2026_10` (1.27 GB), and the catch-up
from 2026-10-31 to 2026-11-02. The runs alternated, sequence first. The catch-up ran with 4 filing
machines, the default before ironcapitaleu/arkad#516. Today's default is 8. These runs do not
measure it.

| Run | Sequence | Parallel |
| --- | -------- | -------- |
| 1 | 94.8 s | 95.9 s |
| 2 | 138.2 s | 93.3 s |
| 3 | 99.2 s | 52.4 s |

- The median parallel run took 93.3 s, against 99.2 s for the sequence. In run 1, the parallel
  run was 1.1 s slower. Its best run took 52.4 s, the time of the Notes step alone.
- In parallel runs 1 and 2, the catch-up made about 2 requests per second while the Notes files
  downloaded, against about 6.5 alone. The cause is not measured.
- The parallel run sent 1 to 6 more requests: the `HEAD` request, and the retries after one and
  two 503 answers in runs 1 and 2. A retry runs the whole filing machine again. Each retry passed.
  No run got a 429. The peak memory was 55 to 426 MB higher.

The full table is in "The Steps at the Same Time" of `sec/design/pipeline/ingestion_paths.md`.

## Consequences

- At best, a run takes the time of its longer step, about half the time of the sequence. In 2 of
  the 3 measured runs, it took about as long as the sequence run.
- The catch-up starts before the run knows that every Notes file exists. The run finds a gap only
  at the end, as a failed run. An operator must rerun on a non-zero exit code.
- In the case of a refused download of the newest period, the Notes summary lists that period as
  `unpublished`, while the run fails. The Notes summary does not know the cut-over date.
- The run sends one `HEAD` request more than the sequence, before any retries.

## Triggers to Revisit

| Trigger | Action |
| ------- | ------ |
| A run passes, and the store later shows a gap before the cut-over date | Check every listed month with a `HEAD` request before any download, or return to the sequence |
| Runs on the production network show no gain over the sequence | Return to the sequence, which needs no `HEAD` request |
| The SEC answers an unpublished file with a status other than 404 | Change rule 2 of the decision. Today any other status stops the run |

## Sources

- STA-435, comments of the owner of 2026-11-06 (the parallel option) and 2026-11-13 (the decision
  and the timing)
- ironcapitaleu/arkad#513 (the parallel run and the timing), ironcapitaleu/arkad#517 and
  ironcapitaleu/arkad#519 (the gap check), ironcapitaleu/arkad#516 (the default of 8 filing
  machines)
- `sec/design/pipeline/ingestion_paths.md`, "Cut-Over Between the Paths" and "The Steps at the Same
  Time"
