# ADR 0022: The Month of the Fiscal Year End Counts February as 29 Days

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

The nearest month end is clear for a day near the start or the end of a month. Two cases need a
rule:

- **February.** The text has no year, so February can have 28 or 29 days. February 14 lies 14 days
  after January 31. It lies 15 days before February 29, and 14 days before February 28.
- **A tie.** A day can lie as far from the end of the previous month as from the end of its own
  month. With a 29-day February, this holds only for day 15 of a 30-day month: `0415` lies 15 days
  from March 31 and 15 days from April 30.

The ticket asks whether `0215` gives January or February. February 15 lies 15 days after January
31, and 14 days before February 29. So February is nearer.

## Decision

1. **February counts 29 days.** The parser dates the text in the leap year 2000, so `0229` is a
   valid fiscal year end. The month uses the same date. `0215` gives 2, and `0214` gives 1.
2. **A tie keeps the month of the day.** `0415` gives 4.

`FiscalYearEnd::month` in `xbrl/src/core/fiscal_year_end.rs` holds the rule.

## Options Considered

Distances in days, counted on 2026-11-15 by hand and checked by the unit tests in Sources.

| Option | `0214` | `0215` | `0415` | Same year as the parser |
| ------ | ------ | ------ | ------ | ----------------------- |
| **A (chosen): 29 days, tie keeps the day's month** | 1 (14 vs 15) | 2 (15 vs 14) | 4 | Yes |
| B: 28 days, tie keeps the day's month | 2 (tie, 14 vs 14) | 2 (15 vs 13) | 4 | No: the parser accepts `0229` |
| C: 29 days, tie takes the previous month | 1 | 2 | 3 | Yes |

- A and B differ only on `0214`. A uses the same leap year as the parser, so one date answers
  both the parse and the month.
- A tie that keeps the month of the day keeps the month the filer wrote. C moves a day the filer
  placed in April to March.

## Consequences

- `0214` gives January. A filer whose year ends on February 14 in a common year gets January,
  although that day lies 14 days from both month ends.
- A change of the rule changes stored silver rows and the changes of the fiscal year they show.

## Open Questions

- The SEC rounds `sub.tsv` `fye` by its own rule. The design states that `sub.tsv` rounds the
  same way for `--09-27` and `--09-01`. No data checks a February day or a tie yet. If the SEC
  rounds such a day another way, the two sources disagree for that company, and this ADR reopens.

## Sources

- `sec/design/pipeline/silver_layer.md`, "`filings`" and the rules for the company data: the
  column and the examples `--09-27` → 9, `0930` → 9, `--09-01` → 8, `0831` → 8.
- [STA-495](https://linear.app/state-machine/issue/STA-495), "Additional Notes": the open question
  on `0215`.
- The unit tests of `xbrl/src/core/fiscal_year_end.rs` (`0214`, `0315`, `0316`, `0414`, `0415`)
  and of `xbrl/src/core/filing.rs` (`0215`).
