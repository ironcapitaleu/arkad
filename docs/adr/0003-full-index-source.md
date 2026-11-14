# ADR 0003: `form.idx` as the Full Index Source

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-14. Amendment 1: Accepted by the project lead on 2026-11-14 |
| Date | 2026-11-13 |
| Deciders | The project lead |
| Ticket | [STA-430](https://linear.app/state-machine/issue/STA-430) |

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
4. **The catch-up starts 7 days before the cut-over date.** It selects the filings with a date
   filed in its range, and the store check skips each stored filing. In the measured range, a
   daily index listed a filing at most 4 days after its date filed in `form.idx`, apart from the
   20-F below. 7 days leave room above that.

## Amendment 1: The Catch-Up Reads `Last Data Received`

Date: 2026-11-14.

**Reason.** The last consequence below, and the last open question, name a gap: a date without rows in `form.idx` can be a day
without filings or a day that the SEC has not added yet. The catch-up counted both as a day without
filings. So a recent date that the SEC had not added yet lost its filings without an alert.

**Decision.** The catch-up reads the header line `Last Data Received` of `form.idx`. A date after
that date, without rows, is pending if it is recent and is not the first date of the run, and fails
otherwise. A date up to that date without rows counts as a day without filings. If the header
names no date, every date without rows counts as a day without filings, as before.

**Consequences.**

- The header of a closed quarter names the last calendar day of the quarter, also if that day is
  a weekend. Measured on 2026-11-14: `2024/QTR1` names March 31, 2024, a Sunday. `2023/QTR3` names
  September 30, 2023, a Saturday. So a closed quarter never leaves a date after its header.
- A header that the parser cannot read turns the check off without an alert.

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
- A filter by date filed misses a filing that the SEC sent inside the range with a date filed
  before the start of the range. The 10-Q above is such a filing: a range that starts on
  2026-10-16 misses it. The daily index path fetches such a filing, because it reads the file of
  the day the SEC sent it.
- A run that reads `form.idx` of a quarter misses a filing that the SEC sent again with a date
  filed in an earlier quarter, as the 20-F above. The Notes files store such a filing only if the
  SEC builds them by date filed. No measurement shows that yet.
- A date without rows in `form.idx` can be a day without filings or a day that the SEC has not
  added yet. The file does not tell the two cases apart by its rows.

## Open Questions

- Decision 4 starts the range 7 days early. It does not select the quarter minus the stored
  filings, because no code lists the stored filings yet. If a measurement shows a lag above 7 days, that selection replaces decision 4.
- No rule yet covers a filing that the SEC sent again with a date filed in an earlier quarter, as
  the 20-F.
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
