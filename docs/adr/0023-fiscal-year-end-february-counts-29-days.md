# ADR 0023: The Month of the Fiscal Year End Counts February as 29 Days

| Field | Value |
| ----- | ----- |
| Status | Proposed. The owner of the ticket confirms |
| Date | 2026-11-15 |
| Deciders | The owner of the ticket |
| Ticket | [STA-495](https://linear.app/state-machine/issue/STA-495) |

## Context

A filing states its fiscal year end as a month and a day with no year: `0930` in the Notes data
set (`sub.tsv` `fye`), or `--09-27` in an instance document (`dei:CurrentFiscalYearEndDate`). A
52-53 week year ends on a weekday near a month end, so the raw day moves from year to year.

The silver `filings` dataset stores `fiscal_year_end_month`: the month of the month end nearest to
the raw day. A change of this month between two filings is a change of the fiscal year. So the
rule decides which stored rows show a change of the fiscal year.

The text has no year, so February can have 28 or 29 days. The choice decides the month of a day
in the middle of February:

- February 14 lies 14 days after January 31. It lies 15 days before February 29, and 14 days
  before February 28.
- February 15 lies 15 days after January 31. It lies 14 days before February 29, and 13 days
  before February 28.

The ticket asks whether `0215` gives January or February. With either length, February is nearer.

[ADR 0024](0024-fiscal-year-end-tie-keeps-the-month.md) holds the rule for a day that lies as far
from two month ends.

## Decision

1. **February counts 29 days.** The parser dates the text in the leap year 2000, so `0229` is a
   valid fiscal year end. The month uses the same date. `0215` gives 2, and `0214` gives 1.

`FiscalYearEnd::month` in `xbrl/src/core/fiscal_year_end.rs` holds the rule.

## Options Considered

Distances in days, counted on 2026-11-15 by hand and checked by the unit tests in Sources.

| Option | `0214` | `0215` | Same year as the parser |
| ------ | ------ | ------ | ----------------------- |
| **A (chosen): 29 days** | 1 (14 vs 15) | 2 (15 vs 14) | Yes |
| B: 28 days | tie (14 vs 14), see ADR 0024 | 2 (15 vs 13) | No: the parser accepts `0229` |

- A and B differ only on `0214`. A uses the same leap year as the parser, so one date answers
  both the parse and the month. With A, February has no tie.

## Consequences

- `0214` gives January. A filer whose year ends on February 14 in a common year gets January,
  although that day lies 14 days from both month ends.
- A change of the rule changes stored silver rows and the changes of the fiscal year they show.

## Open Questions

- The SEC rounds `sub.tsv` `fye` by its own rule. The design states that `sub.tsv` rounds the
  same way for `--09-27` and `--09-01`. No data checks a February day yet. If the SEC rounds such
  a day another way, the two sources disagree for that company, and this ADR reopens.

## Sources

- `sec/design/pipeline/silver_layer.md`, "`filings`" and the rules for the company data, read
  2026-11-15: the column and the examples `--09-27` → 9, `0930` → 9, `--09-01` → 8, `0831` → 8.
- [STA-495](https://linear.app/state-machine/issue/STA-495), "Additional Notes", read 2026-11-15:
  the open question on `0215`.
- The unit tests of `xbrl/src/core/fiscal_year_end.rs` (`0214`, `0229`) and of
  `xbrl/src/core/filing.rs` (`0215`), run 2026-11-15.
