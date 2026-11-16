# ADR 0027: Debt Maturity Buckets Are Concepts at the Fiscal Year End

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-503](https://linear.app/state-machine/issue/STA-503) |

## Context

A maturity ladder gives the principal of long-term debt that falls due in each of the next five
fiscal years and after them. US GAAP tags each bucket with its own concept, such as
`LongTermDebtMaturitiesRepaymentsOfPrincipalInYearTwo`. Each fact is an instant at the balance
sheet date.

The standardized layer stores one row per entity, concept, period, unit, and dimensions in
`standardized_facts`. The period key is `period_end` and `quarters`. The ladder needs a form in
this key. The form binds the `xbrl` concepts, the alias lists, the series step, the gap check, and
the rows of `standardized_facts`. A change of the form later rewrites stored rows.

Terms in this ADR:

- **Bucket:** one span of the ladder, such as year two.
- **Fiscal-year bucket:** a bucket that counts in fiscal years from the fiscal year end.
- **Rolling bucket:** a bucket that counts in 12-month spans from the balance sheet date.

## Decision

1. **Each bucket is a standard concept, and each value is an instant.** The disclosure
   `DebtMaturities` holds `DebtDueInYearOne` to `DebtDueInYearFive`, `DebtDueAfterYearFive`, and
   the total `LongTermDebtIncludingCurrentMaturities`. A value is a row with `period_end` at the
   balance sheet date and `quarters` 0.
2. **Only the period of report of an annual report gives a bucket value.** The standardize filing
   step drops a candidate of a bucket unless its instant is the period of report of a filing with
   the fiscal period `FY`. An interim ladder stays an as-is fact. The total is a balance sheet
   value, so it gets a candidate at every balance sheet date.
3. **The rolling concept is the second alias of each bucket.** At a fiscal year end, a
   fiscal-year bucket and a rolling bucket hold the same amount.
4. **A bucket that the filer does not tag has no row.** No rule derives it, and silver never
   writes a zero for it.

## Options Considered

Measured on 2026-11-15 in the SEC frames API, frame `CY2024Q4I`, facts without dimensions. The
frame holds the instant nearest 2024-12-31 for each filer. The SEC submissions API gives the form
and the report date of the accession of each value. An annual ladder is a value from a 10-K,
10-KT, 20-F, or 40-F, or an amendment of one, at its report date.

| Option | Rows per ladder | New column | Fits the series key | Fits the existing checks |
| ------ | --------------- | ---------- | ------------------- | ------------------------ |
| **A (chosen): bucket in the concept, instant** | 7 | none | yes | yes, with one gap rule |
| B: one concept, the bucket as a future period | 7 | none | no | no |
| C: one concept, the bucket as a new key column | 7 | `bucket` | yes | no |

- A keeps the period of the filing. The series of `DebtDueInYearOne` gives the debt due in the
  next year, for each year.
- B stores a future duration, such as 2026-01-01 to 2026-12-31 for year two of the 2024 ladder.
  Each new 10-K gives another value for the same future period, so dedup takes it for a
  restatement. The gap check and the suspicious period check read the period as a reported
  period. "After year five" has no end.
- C adds a key column to `standardized_facts` that only seven concepts use. Every reader and the
  dedup rules change.
- An interim ladder has no stable meaning. In an interim report, a fiscal-year bucket counts
  from the fiscal year end, and a rolling bucket counts from the balance sheet date. 86 filers in
  the frame tag `LongTermDebtMaturitiesRepaymentsOfPrincipalRemainderOfFiscalYear`, which only an
  interim report has. The frame holds 678 bucket values of interim reports. Decision 2 leaves
  these values as-is.
- In the annual ladders, the rolling aliases add 52 filers that tag no fiscal-year bucket. 1,582
  filers tag a fiscal-year bucket. 1,634 tag one after the aliases.
- 1,082 bucket values of the annual ladders are zero. Decision 4 keeps a tagged zero apart from a bucket
  the filer did not tag.

## Consequences

- The ladder adds seven concepts and no column. `CanonicalElementKind` holds 59 standard concepts.
- The gap check expects only fiscal year ends for the six buckets. Without the rule, every 10-Q
  filer gets three `Gap` findings per bucket and year. The trait `Timed` marks the buckets.
- A filter after `resolve_aliases` reads the `Filing` and `CanonicalElementKind::cadence`, which
  the middle macro generates from `Timed`. `resolve_aliases` and the `Taxonomy` trait do not
  change.
- A reader cannot tell "no debt" from "schedule not tagged". It shows a bucket with no row as "not
  reported in machine-readable form".
- The decision does not cover interim ladders, ladders with finance leases, ladders per legal
  entity, or the notes on `DebtInstrumentAxis`. These stay as-is facts.

## Open Questions

- A demand for interim ladders reopens decision 2.

## Sources

- SEC frames API, `https://data.sec.gov/api/xbrl/frames/us-gaap/<concept>/USD/CY2024Q4I.json`,
  one request per concept, read on 2026-11-15. Each frame holds one value per filer, without
  dimensions.
- SEC submissions API, `https://data.sec.gov/submissions/CIK<cik>.json`, one request per filer
  with a bucket, read on 2026-11-15: the form and the report date of each accession.
- `xbrl/taxonomy/us-gaap/2026/stm/us-gaap-stm-sfp-dbo1-cal-2026.xml`: `LongTermDebt` is the
  parent of the six buckets.
- `xbrl/taxonomy/us-gaap/2026/elts/us-gaap-2026.xsd`: every bucket concept has the period type
  `instant`.
- `xbrl/DESIGN.md`, "Debt Maturities".
