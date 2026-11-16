# ADR 0036: No Parser for the Company Concept and Frames APIs

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-16 ([STA-529 comment `c1b413c2`](https://linear.app/state-machine/issue/STA-529#comment-c1b413c2)) |
| Date | 2026-11-16 |
| Deciders | The project lead |
| Ticket | [STA-529](https://linear.app/state-machine/issue/STA-529) |

## Context

The facts of the pipeline come from two sources. The Financial Statement and Notes data set
serves the backfill. The XBRL instance document of each new filing from the daily index serves the
rest.
`xbrl::adapter::edgar::notes` and `xbrl::adapter::edgar::instance` parse them. The
`companyfacts` JSON is a cross-check only, because it drops every value with a dimension
([`ingestion_paths.md`](../../sec/design/pipeline/ingestion_paths.md), "Sources").

The SEC also serves XBRL facts through three JSON APIs on `data.sec.gov`:

1. **Company Facts** (`/api/xbrl/companyfacts/CIK{n}.json`): every concept of one company.
2. **Company Concept** (`/api/xbrl/companyconcept/CIK{n}/{taxonomy}/{concept}.json`): one concept
   of one company.
3. **Frames** (`/api/xbrl/frames/{taxonomy}/{concept}/{unit}/{period}.json`): one concept of many
   companies for one period.

All three serve data points with the same 9 keys. Only the JSON envelope differs.

`xbrl::adapter::edgar` had a module for Company Concept and one for Frames. Each held a TODO and
no parser. The choice binds each later ticket that adds a parser to `xbrl::adapter::edgar` or a
request kind to `sec` for one of these two APIs.

## Decision

1. **`xbrl::adapter::edgar` holds no parser for the Company Concept API or the Frames API.** The
   two placeholder modules are deleted.
2. **`sec` fetches no Company Concept body and no Frames body.**

## Options Considered

Checked on 2026-11-16 on `main` at `d0af124`.

| Option | Facts that only this API gives to the pipeline | Request kinds of `sec` that fetch the API |
| ------ | ---------------------------------------------- | ----------------------------------------- |
| **A: no parser for either API (chosen)** | Not applicable | Not applicable |
| B: add a Company Concept parser | 0: the Notes data set, the instance documents, and Company Facts hold every concept of the company | 0 |
| C: add a Frames parser | Facts of other companies for one concept, which no state of `sec` reads | 0 |

- A Company Concept body is a subset of the Company Facts body of the same company, and Company
  Facts is a cross-check only.
- A Frames body serves one concept across companies. Each state of `sec` works on one company,
  one filing, or one Notes period at a time. The Notes data set already holds every company of a
  period.

## Consequences

- `xbrl::adapter::edgar` holds no module that parses nothing.
- The `frame` and `quarter` modules stay. They parse the `frame` field of a Company Facts data
  point, not the Frames API.
- The decision says nothing about Company Facts. Its parser stays while the cross-check reads it.
- A design doc can still cite the Frames API as a source of a measurement. That use reads no body
  through `xbrl`.

## Open Questions

- The decision reopens if a state of `sec` must read one concept across many companies, and the
  stored Notes rows and instance documents cannot serve that read.

## Sources

- `sec/src/lib/shared/request/implementations/sec_request/constants.rs` at `d0af124`: the URL
  prefixes of the request kinds of `sec`, read on 2026-11-16.
- SEC, "EDGAR Application Programming Interfaces",
  `https://www.sec.gov/search-filings/edgar-application-programming-interfaces`: the paths of the
  three APIs and the keys of a data point, read on 2026-11-16.
