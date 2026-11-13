# ADR 0003: `form.idx` as the Full Index Source

| Field | Value |
| ----- | ----- |
| Status | Proposed. The lead confirms |
| Date | 2026-11-13 |
| Deciders | The project lead |

## Context

The catch-up of `backfill` reads 30 to 60 days. Today it sends one daily index request per day.
The SEC also publishes one full index per quarter, in two files:

- `www.sec.gov/Archives/edgar/full-index/YYYY/QTRn/form.idx` lists every filing of the quarter.
- `www.sec.gov/Archives/edgar/full-index/YYYY/QTRn/xbrl.idx` lists only the filings with XBRL.

Each row of both files holds the form, the company name, the CIK, the date filed, and the file
name. The catch-up needs one of the two files as its source. `daily_filings` later reads the same
file (see
[`repository_listing.md`](../../storage/design/data_model/repository_listing.md#3-daily_filings-full-index-minus-stored-filings)).
So the choice binds both runs.

In this document, a selected filing has a form in `SELECTED_FORMS` or an amendment of one. Each
count takes an accession number once.

## Decision

1. **The catch-up of `backfill` reads `form.idx`.** `daily_filings` reads the same file when it
   moves to the full index.
2. **The store keeps each full index under one key per quarter:**
   `sec/full-index/YYYY/QTRn/form.idx`. A later run replaces the file.
3. **One parser reads the daily index and `form.idx`.** Both files have the same fixed-width
   layout. Only the date filed differs: `20260930` in the daily index and `2026-09-30` in
   `form.idx`.

## Options Considered

Measured on 2026-11-13 against the SEC, for 2026 Q3 and for 2026 Q4 through 2026-11-12.

| | **`form.idx`** | `xbrl.idx` |
| --- | --- | --- |
| Size, 2026 Q3 | 47.2 MB, or 4.8 MB as `form.gz` | 5.1 MB |
| Selected filings, 2026 Q3 | 5,919 | 5,878. It lacks 41 |
| Selected filings, 2026 Q4 through 2026-11-12 | 4,022 | 4,009. It lacks 13 |
| Selected filings that only this file lists | 54 | 0 |
| Last date filed in the Q4 file | 2026-11-12 | 2026-11-12 |
| Layout | Fixed width, as the daily index | Fields with `\|` between them |

The daily indexes from 2026-09-21 through 2026-11-12 hold 4,318 selected filings. `form.idx` of Q3
and Q4 holds 4,317 selected filings with a date filed in that range, and the daily indexes hold
each of them.

Reasons for `form.idx`:

- **It lists every filing that the daily index path selects.** The 54 filings that `xbrl.idx` lacks
  have no XBRL. The daily index path stores the folder index of such a filing, and a later run
  skips the filing with no request. With `xbrl.idx`, the store holds no record of these filings.
- **It needs no second parser.** `xbrl.idx` has another layout.
- **Its size fits the catch-up.** The catch-up reads one or two quarters per run.

## Consequences

- The catch-up sends one request per quarter in place of one request per day.
- A run stores up to 47 MB per quarter under one key. The SHA-256 in the `inputs` of an instance
  names bytes that a later run replaces. The daily index has the same gap.
- `form.idx` lists a filing once, with the date filed of the last time the SEC sent it. A daily
  index lists a filing on the day the SEC sent it. Two differences follow, both seen in the
  measured range:
  - The daily index of 2026-11-09 lists a 20-F with the date filed 2026-03-31
    (`0001628280-26-022148`). `form.idx` lists it in Q1 only. So a run that reads `form.idx` does
    not see that the SEC sent an old filing again. This is the one filing that only the daily
    indexes hold.
  - The daily index of 2026-10-16 lists a 10-Q with the date filed 2026-10-16. The daily index of
    2026-10-19 lists it again with 2026-10-15 (`0001437749-26-032746`). `form.idx` lists it once,
    with 2026-10-15.
- A filter by date filed misses a filing whose date filed lies before the start of the range,
  if the SEC sent it inside the range. The 20-F above is such a filing. The daily index
  path fetches them, because it reads the file of the day the SEC sent the filing. The Notes
  files cover a date filed before the cut-over date only if the SEC builds them by date filed,
  and no measurement shows that yet.
- A date without rows in `form.idx` can be a day without filings or a day that the SEC has not
  added yet. The file does not tell the two cases apart by its rows.

## Open Questions

- The catch-up filters by date filed, so it needs a rule for a filing with a date filed before
  the start of its range. Two options: start the range some days before the cut-over date and
  let the store check skip the stored filings, or select the quarter minus the stored filings, as
  `daily_filings` does. The change that makes the catch-up read `form.idx` picks one.
- For `daily_filings`, one download of 47 MB per run can cost too much. `form.gz` holds the same
  rows in 4.8 MB. No measurement of the daily run exists yet.
- The header line `Last Data Received` names the last date of the file. No code reads it yet. It
  can tell a day without filings from a day that the SEC has not added.

A change of the layout of `form.idx`, or a lag longer than that of the daily index, reopens this
decision.

## Sources

- The files `form.idx` and `xbrl.idx` of `full-index/2026/QTR3` and `full-index/2026/QTR4`, and
  the daily indexes `daily-index/2026/QTR3/form.YYYYMMDD.idx` and
  `daily-index/2026/QTR4/form.YYYYMMDD.idx` from 2026-09-21 through 2026-11-12, downloaded on
  2026-11-13.
- [`extraction_overview.md`](../../sec/design/pipeline/extraction_overview.md), "Endpoints".
- [`repository_listing.md`](../../storage/design/data_model/repository_listing.md), "Bookmark or
  Full Index" and "Assumptions and Triggers".
