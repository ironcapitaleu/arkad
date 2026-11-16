# ADR 0036: `xbrl` Parses Only the Company Facts API of the SEC JSON APIs

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-16 |
| Deciders | The project lead |
| Ticket | [STA-529](https://linear.app/state-machine/issue/STA-529) |

## Context

The SEC serves XBRL facts through three JSON APIs on `data.sec.gov`:

1. **Company Facts** (`/api/xbrl/companyfacts/CIK{n}.json`): every concept of one company.
2. **Company Concept** (`/api/xbrl/companyconcept/CIK{n}/{taxonomy}/{concept}.json`): one concept
   of one company.
3. **Frames** (`/api/xbrl/frames/{taxonomy}/{concept}/{unit}/{period}.json`): one concept of many
   companies for one period.

All three serve data points with the same 9 keys. Only the JSON envelope differs.

`xbrl::adapter::edgar` had one module per API. The modules for Company Concept and Frames held a
TODO and no parser. The choice binds each later ticket that adds a parser to `xbrl::adapter::edgar`
or a request kind to `sec` for one of these APIs.

## Decision

1. **`xbrl::adapter::edgar` parses the Company Facts API and no other SEC JSON API.** The
   placeholder modules for Company Concept and Frames are deleted.
2. **`sec` fetches no Company Concept body and no Frames body.**

## Options Considered

Checked on 2026-11-16 on `main` at `d0af124`.

| Option | Facts that only this API gives to `sec` | Request kinds of `sec` that fetch the API | Parser code in `xbrl` |
| ------ | --------------------------------------- | ----------------------------------------- | --------------------- |
| **A: Company Facts only (chosen)** | Not applicable | 1: `SEC_COMPANY_FACTS_URL_PREFIX` | `company_facts` |
| B: add Company Concept | 0: Company Facts holds every concept of the company | 0 | 3-line placeholder |
| C: add Frames | Facts of other companies, which no state of `sec` reads | 0 | 3-line placeholder |

Source of the request kinds: the URL prefixes in
`sec/src/lib/shared/request/implementations/sec_request/constants.rs`.

- A Company Concept body is a subset of the Company Facts body of the same company. Each fact it
  holds is already parsed.
- A Frames body serves one concept across companies. Each state of `sec` works on one company or
  one filing at a time.

## Consequences

- `xbrl::adapter::edgar` holds no module that parses nothing.
- The `frame` and `quarter` modules stay. They parse the `frame` field of a Company Facts data
  point, not the Frames API.
- A design doc can still cite the Frames API as a source of a measurement. That use reads no body
  through `xbrl`.

## Open Questions

- The decision reopens if a state of `sec` must read one concept across many companies, such as a
  peer comparison. Frames then serves that read with one request per concept and period.
