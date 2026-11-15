# ADR 0023: A Fiscal Year End as Far From Two Month Ends Keeps Its Month

| Field | Value |
| ----- | ----- |
| Status | Proposed. The owner of the ticket confirms |
| Date | 2026-11-15 |
| Deciders | The owner of the ticket |
| Ticket | [STA-495](https://linear.app/state-machine/issue/STA-495) |

## Context

The silver `filings` dataset stores `fiscal_year_end_month`: the month of the month end nearest to
the raw fiscal year end, such as `0930` or `--09-27`. A change of this month between two filings
is a change of the fiscal year. So the rule decides which stored rows show a change of the fiscal
year.

A day can lie as far from the end of the previous month as from the end of its own month. Such a
day is a tie. With February at 29 days ([ADR 0022](0022-fiscal-year-end-february-counts-29-days.md)),
a tie occurs only on day 15 of a 30-day month: April, June, September, or November. For example,
`0415` lies 15 days from March 31 and 15 days from April 30.

## Decision

1. **A tie keeps the month of the day.** `0415` gives 4.

`FiscalYearEnd::month` in `xbrl/src/core/fiscal_year_end.rs` holds the rule.

## Options Considered

Distances in days, counted on 2026-11-15 by hand and checked by the unit tests in Sources.

| Option | `0415` | `0915` |
| ------ | ------ | ------ |
| **A (chosen): the month of the day** | 4 | 9 |
| B: the previous month | 3 | 8 |

- A keeps the month the filer wrote. B moves a day the filer placed in April to March.

## Consequences

- Four days of the year are ties: `0415`, `0615`, `0915`, and `1115`.
- A change of the rule changes stored silver rows and the changes of the fiscal year they show.

## Open Questions

- No data checks how the SEC rounds a tie in `sub.tsv` `fye`. If the SEC rounds a tie another way,
  the two sources disagree for that company, and this ADR reopens.

## Sources

- `sec/design/pipeline/silver_layer.md`, "`filings`" and the rules for the company data, read
  2026-11-15: the column `fiscal_year_end_month`.
- The unit tests of `xbrl/src/core/fiscal_year_end.rs` (`0414`, `0415`), run 2026-11-15.
