# ADR 0030: The Segment Reconciliation Runs per Report With an Unreconciled Line

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-502](https://linear.app/state-machine/issue/STA-502/design-segments-in-the-standardized-layer) |

## Context

A user wants to see that the segments add up to the consolidated value. Under ASC 280, the
segments, the inter-segment eliminations, and the corporate items add up to the consolidated value.
Filers tag the eliminations and the corporate items on `srt:ConsolidationItemsAxis`, but often they
tag none.

ADR 0029 makes each segment value a standardized fact with its dimensions. This ADR decides how a
reader gets the reconciliation from those values.

Terms in this ADR:

- **Business-segment axis:** `BusinessSegments` or `SegmentReportingInformationBySegment`.
- **Group:** the values of one filing, one standard concept, one period, and one unit, with two or
  more segments on `BusinessSegments`. The measurement counts only this axis.
- **Adds up:** the difference is within 0.1% of the larger absolute value.
- **Role:** segment, elimination, or corporate, by the member of `ConsolidationItems`
  (`xbrl/DESIGN.md`, "Reconciliation").

The decision binds the read type of the reconciliation in `core`, the silver writer, and the gold
load.

## Decision

1. **The reconciliation runs on the candidates of one filing, over the business-segment axes.** The
   segments, the eliminations, the corporate items, and the consolidated value come from one report.
2. **An unreconciled line holds the rest:** consolidated − segments − eliminations − corporate items,
   each with the sign of the filing. Its role rule ID is `core/segment/unreconciled@1`. A reported
   line keeps the rule of its candidate, and an elimination or corporate line carries the role rule
   ID `core/segment/elimination@1` or `core/segment/corporate@1` next to it. If the consolidated
   candidate or a candidate with a role is blocked, the reconciliation has no unreconciled line.
   Otherwise a blocked amount looks like an untagged item.
3. **Silver stores no line of the reconciliation.** A read type computes the lines, as the
   statement output type computes the other lines.
4. **An unreconciled line other than zero gives no finding.**
5. **If a segment is filed in both forms, the value without `ConsolidationItems` counts.**

## Options Considered

Measured on 2026-11-15 on `2026_10_notes.zip` of the SEC, 978 10-K and 10-Q filings.

| Group adds up | Revenue | Operating income | Assets |
| ------------- | ------- | ---------------- | ------ |
| Groups | 1,817 | 851 | 479 |
| Segments alone | 1,246 (69%) | 204 (24%) | 131 (27%) |
| With the tagged eliminations and corporate items | 243 (13%) | 189 (22%) | 222 (46%) |
| Not at all | 328 (18%) | 458 (54%) | 126 (26%) |
| No elimination or corporate item tagged | 1,014 (56%) | 509 (60%) | 201 (42%) |

| Option | Scope of the values | Line for the rest | Stored |
| ------ | ------------------- | ----------------- | ------ |
| **A: per report, unreconciled line, read time (chosen)** | One filing | Yes | No |
| B: per series, after `core/dedup/newest-filing@1` | Many filings | Yes | No |
| C: per report, a check with a finding | One filing | No | Findings |
| D: A, with the unreconciled line stored as a derived fact | One filing | Yes | Yes |

- A shows every group, including the 54% of operating income groups that do not add up. The line
  tells a reader where the tagged values stop.
- B mixes filings. After a reorganization, the newest consolidated value meets segments of older
  filings under other names. ASC 280 recasts the earlier years in one report, so the values of one
  report fit together.
- C gives a finding for most filers, because 458 of 851 operating income groups (54%) do not add
  up.
- D needs a member for the line in `dimensions` that no filer uses, and a stored copy goes stale
  when a candidate changes. The other lines of the tree follow the same rule
  (`xbrl/DESIGN.md`, "\"Other\" Lines").

Facts behind decisions 2 and 5:

- Of 590 revenue values with an elimination role alone, 444 are negative, 88 positive, and 58 zero.
  A flip of the sign of the positive ones closes 7 more groups. So each value keeps its sign.
- Plain segment values close 1,489 revenue groups. Values with `OperatingSegments` close 1,430.

## Consequences

- A reader sees the unreconciled line next to the tagged roles. Only the unreconciled line has no
  candidate rule, so a reader tells it from a reported line.
- Members of a company on `ConsolidationItems`, such as `CorporateAndEliminations`, have no role.
  Their amount lands in the unreconciled line.
- A value of a segment with a second pair, such as its inter-segment revenue, has no role. A filer
  can report the segment revenue with or without it, so the unreconciled line absorbs the
  difference.
- The reconciliation runs only on the business-segment axes. Members on `Geographical` and
  `ProductOrService` can overlap, such as `US`, `NonUs`, and `Americas` in one filing. A sum then
  counts an amount twice, so these axes get no reconciliation.
- Of 456 cases of one filing, revenue, period, and unit with one segment, 425 (93%) equal the
  consolidated value.
- A series view over many reports shows no reconciliation.
- 343 of 6,819 keys with a corporate role carry both corporate members, and 49 of them carry
  equal values. The corporate role then likely counts one amount twice, and the unreconciled line
  shows it as a negative amount.

## Open Questions

- A finding for a large unreconciled line: if users ask for one, a later design sets a threshold
  from a measurement over more Notes files.
- The measurement covers one month of filings. A run over the files of a full year can move the
  shares, but it does not change the options.

## Sources

- `measure_segments.py` and `measure_segments_operating_first.py` on
  `https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2026_10_notes.zip`,
  run on 2026-11-15. The scripts and their output are attached to the ticket.
- `xbrl/DESIGN.md`, "Segment Values" and "\"Other\" Lines".
- The members of `ConsolidationItems`: `xbrl/taxonomy/us-gaap/2026/elts/us-gaap-2026.xsd` and
  `xbrl/taxonomy/srt/2026/elts/srt-2026.xsd`.
- ADR 0029.
