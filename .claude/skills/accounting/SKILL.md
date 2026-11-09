---
name: accounting
description: >
  Use when a task needs financial accounting knowledge: the elements of financial statements, the
  identities between them, US GAAP and IFRS and their differences, balances and flows, sign
  conventions, fiscal years and fiscal periods, 52-53 week years, transition periods, periodic
  reports of the SEC and of EU issuers, restatements, and which values add over periods. Also use
  when the user asks "does this balance", "how does IFRS treat X", "is this a restatement", "what
  is Q4", or "which period is this".
version: 0.2.0
---

# Accounting

## Purpose

This skill holds financial accounting knowledge for this project: the frameworks, the identities
between the statements, periods, and reporting calendars. It covers US GAAP and IFRS.

## Frameworks

| | US GAAP | IFRS |
| --- | --- | --- |
| Standard setter | FASB | IASB |
| Standards | Accounting Standards Codification (ASC) | IFRS and IAS standards |
| Elements | Concepts Statement 8, Chapter 4 (2021): ten elements. It replaced SFAC 6 | Conceptual Framework (2018): five elements |
| Used by | US domestic SEC filers | EU listed groups, most other countries, and foreign issuers at the SEC |

The ten US GAAP elements are assets, liabilities, equity, investments by owners, distributions to
owners, comprehensive income, revenue, expenses, gains, and losses. SFAC 6 (1985) defined them.
Concepts Statement 8, Chapter 4 (December 2021) replaced SFAC 6. It keeps the ten elements and
rewords their definitions. `references/sfac6-elements.md` still quotes the SFAC 6 text. The five IFRS elements are assets, liabilities, equity, income,
and expenses. IFRS income includes gains, and IFRS expenses include losses.

## Identities

These hold under both frameworks:

1. **Balance sheet:** Assets = Liabilities + Equity
2. **Comprehensive income:** Comprehensive Income = Revenue − Expenses + Gains − Losses
3. **Changes in equity:** ΔEquity = Comprehensive Income + Investments by Owners −
   Distributions to Owners
4. **Net income:** Comprehensive Income = Net Income + Other Comprehensive Income
5. **Cash flow:** Operating CF + Investing CF + Financing CF + Effect of exchange rates
   = ΔCash and cash equivalents

Identities 1 to 3 follow from the elements. Identity 4 comes from ASC 220 and IAS 1 (IFRS 18 from 2027). Identity 5
comes from ASC 230 and IAS 7. Under US GAAP, the cash total of identity 5 includes restricted cash.

## US GAAP and IFRS Differences

| Topic | US GAAP | IFRS |
| --- | --- | --- |
| Name of net income | net income | profit or loss |
| Name of the balance sheet | balance sheet | statement of financial position |
| Inventory cost | LIFO allowed | LIFO not allowed (IAS 2) |
| Impairment reversal | not allowed for assets held and used | allowed, except for goodwill (IAS 36) |
| Development costs | expensed, with few exceptions | capitalized if criteria are met (IAS 38) |
| Lessee accounting | operating lease: one straight-line lease cost, payments in operating cash flow (ASC 842) | every lease: depreciation and interest, principal payments in financing cash flow (IFRS 16) |
| Interest paid in cash flow | operating | operating or financing (IAS 7) |

Under both frameworks, a lessee puts its leases on the balance sheet and can exempt short-term
leases. IFRS 16 also exempts leases of low-value assets.

IFRS 18 replaces IAS 1 for annual periods that begin on or after 1 January 2027. Early application
is allowed. It applies retroactively, so a report under IFRS 18 also changes its comparatives. It
adds required subtotals to the statement of profit or loss and changes the cash flow
classification of interest and dividends.
Check which standard applies to the period of a report.

## Balances and Flows

- A balance sheet value covers an instant: the balance on one date.
- An income statement or cash flow value covers a duration: the flow over a period.
- A value per share and a share count do not add over periods. A ratio does not add either.

## Sign Conventions

- Expenses and losses are positive values that reduce income.
- Keep the sign the report gives. A negative value can be correct, for example a net loss.

## Fiscal Years and Fiscal Periods

- A fiscal year ends on a date the company chooses. The fiscal year end is a month and a day,
  for example 30 September.
- A 52-53 week year ends on the same weekday each year, for example the last Saturday of
  September. Its end moves by up to 6 days from the end of the month, and some years have 53
  weeks.
- A company that changes its fiscal year end reports a transition period between the old end and
  the new end. At the SEC, the transition period is shorter than a year. Under IFRS (IAS 1.36), it
  can be shorter or longer than a year.
- A quarterly reporter has the periods `Q1` to `Q4` and `FY`. A half-yearly reporter has `H1`,
  `H2`, and `FY`.
- A company often does not report a fourth quarter or a second half on its own. It is the fiscal
  year less the earlier periods. This works only for values that add over periods.

## Reporting Calendars

| Regime | Annual report | Interim report |
| --- | --- | --- |
| US SEC, domestic filer | `10-K` | `10-Q` for `Q1` to `Q3`, with the quarter and the year to date |
| US SEC, foreign private issuer | `20-F`, or `40-F` for a Canadian issuer | none required |
| EU regulated market | annual financial report | half-yearly financial report |

- At the SEC, `10-KT` and `10-QT` report a transition period. A suffix `/A` marks an amendment,
  for example `10-K/A`.
- A `10-Q` reports the cash flow statement for the year to date only.
- An EU listed group reports its consolidated accounts under IFRS as adopted by the EU. Its
  half-yearly report covers `H1`.
- A foreign private issuer at the SEC can use IFRS as issued by the IASB with no reconciliation to
  US GAAP.

## Restatement and Repeated Comparative

A later report repeats the values of earlier periods as comparatives. Two copies of one value can
agree or differ:

- **Repeated comparative:** the copies agree within the precision of the less precise copy.
- **Restatement:** the copies differ beyond that precision. The later value replaces the earlier
  one. The cause is a correction of an error, a change in accounting policy, or a re-presentation,
  for example of a discontinued operation (ASC 250, IAS 8, ASC 205-20, IFRS 5).

The latest report holds the current value. A read as of an earlier date uses the latest report
published by that date.

## Critical Invariants

- A balance sheet that does not balance with values as reported is an error in the report.
- Never add per-share values, share counts, or ratios over periods.
- Never compare an instant with a duration.
- Never compare a US GAAP value with an IFRS value without checking the differences above.

## Authoritative Sources

- **SFAC 6 definitions (superseded):** `references/sfac6-elements.md`
- **FASB Concepts Statements:** `https://www.fasb.org/standards/concepts-statements`
- **US GAAP:** FASB Accounting Standards Codification, `https://asc.fasb.org`
- **IFRS:** IFRS Foundation, `https://www.ifrs.org/issued-standards/`
- **SEC forms:** `https://www.sec.gov/forms`

## Staleness Check

Standards change. A standard setter can replace a document, as FASB replaced SFAC 6 with Concepts
Statement 8, Chapter 4. Each reference file names its `source`, `last-verified`, and
`update-frequency` in its header.

1. When the skill loads, read the header of each reference file the task uses.
2. If `last-verified` is more than 12 months old, or the header has `superseded-by`, check the
   source before you rely on the file:
   - US GAAP: the FASB Concepts Statements and the Accounting Standards Updates.
   - IFRS: the issued standards of the IFRS Foundation and their effective dates.
3. If the source still holds, set `last-verified` to today.
4. If a newer document replaced the source, tell the user. Propose to:
   - add `superseded-by` to the header and update the file from the new document,
   - update every statement of this skill that names the old document,
   - search the repository for the old name, for example `git grep -n "SFAC 6"`, and list the
     hits for a follow-up ticket.
5. Store the text of a standard as the standard writes it. Never rewrite it from memory.

Check the "US GAAP and IFRS Differences" table in the same way. A new standard, such as IFRS 18,
can change a row.

## Self-Improvement

If the user corrects an accounting fact, or a task finds a new US GAAP and IFRS difference, ask
the user whether to add it to this skill.
