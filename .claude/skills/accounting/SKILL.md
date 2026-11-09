---
name: accounting
description: >
  Use when a task needs accounting knowledge that does not depend on XBRL: the elements of
  financial statements, the SFAC 6 and US GAAP identities, instants and durations, sign conventions,
  fiscal years and fiscal periods, 52-53 week years, transition periods, the SEC report forms,
  restatements and repeated comparatives, and which values add over periods. Also use when the
  user asks "does this balance", "is this a restatement", "what is Q4", or "which period is this".
version: 0.1.0
---

# Accounting

## Purpose

This skill holds the accounting background that the code of this project relies on. It is
independent of XBRL and of any taxonomy. The `xbrl-standardization` skill holds the rules of the
`xbrl` crate and builds on this one.

## Identities

SFAC 6 (Statement of Financial Accounting Concepts No. 6, 1985) defines ten elements of financial
statements. Three identities follow from them:

1. **Balance sheet:** Assets = Liabilities + Equity
2. **Comprehensive income:** Comprehensive Income = Revenue − Expenses + Gains − Losses
3. **Changes in equity:** ΔEquity = Comprehensive Income + Investments by Owners − Distributions
   to Owners

SFAC 6 defines no net income and no cash flow. US GAAP adds two identities:

4. **Net income (ASC 220):** Comprehensive Income = Net Income + Other Comprehensive Income
5. **Cash flow (ASC 230):** Operating CF + Investing CF + Financing CF + Effect of exchange rates
   = Δ(Cash, cash equivalents, and restricted cash)

The definitions of the ten elements are in `references/sfac6-elements.md`.

## Instants and Durations

- A balance sheet value covers an instant: the balance on one date.
- An income statement or cash flow value covers a duration: the flow over a period.
- A value per share and a share count do not add over periods. A ratio does not add either.

## Sign Conventions

- Expenses and losses are positive values that reduce income.
- Keep the sign the filer reports. A negative value can be correct, for example a net loss.

## Fiscal Years and Fiscal Periods

- A fiscal year ends on a date the company chooses. The fiscal year end is a month and a day,
  for example `0930`.
- A 52-53 week year ends on the same weekday each year, for example the last Saturday of
  September. Its end moves by up to 6 days from the end of the month, and some years have 53
  weeks.
- A company that changes its fiscal year end reports a transition period between the old end and
  the new end. The transition period is shorter than a year.
- The fiscal periods of an SEC filer are `Q1` to `Q3` and `FY`. Other regimes report half years,
  for example `H1`.

## SEC Report Forms

| Form | Filer | Reports |
| --- | --- | --- |
| `10-K` | domestic | the fiscal year |
| `10-Q` | domestic | `Q1` to `Q3`: the quarter and the year to date |
| `10-KT`, `10-QT` | domestic | a transition period |
| `20-F` | foreign private issuer | the fiscal year |
| `40-F` | Canadian issuer under the multijurisdictional system | the fiscal year |

A suffix `/A` marks an amendment, for example `10-K/A`. An amendment is filed later than the
report it amends.

A domestic filer files no 10-Q for the fourth quarter. The 10-K reports the fiscal year, so the
fourth quarter is the fiscal year less the first nine months. This works only for values that add
over periods. A 10-Q reports the cash flow statement for the year to date only.

## Restatement and Repeated Comparative

A later report repeats the values of earlier periods as comparatives. Two copies of one value can
agree or differ:

- **Repeated comparative:** the copies agree within the precision of the less precise copy.
- **Restatement:** the copies differ beyond that precision. The later value corrects the earlier
  one.

The latest report holds the current value. A read as of an earlier date uses the latest report
filed by that date.

## Critical Invariants

- A balance sheet that does not balance with values as filed is an error in the filing.
- Never add per-share values, share counts, or ratios over periods.
- Never compare an instant with a duration.

## Authoritative Sources

- **SFAC 6 definitions:** `references/sfac6-elements.md`
- **SEC forms:** `https://www.sec.gov/forms`
