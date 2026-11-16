# ADR 0029: Segment Values Keep Their Dimensions and Stay Out of the Tree

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-502](https://linear.app/state-machine/issue/STA-502/design-segments-in-the-standardized-layer) |

## Context

The as-is layer keeps every fact with its dimensions, a set of axis and member pairs. The
standardized layer holds only consolidated values: in tiers 1 and 2, a fact with dimensions gives
no candidate (`xbrl/src/core/alias.rs`). The lead decided on 2026-10-18 to show the segments of a
company per report, and silver must hold them in the same form for every source.

Segment data differs from the consolidated statements in three ways:

- Each company names its own segments. In the S&P 500, 86% of the members on the business-segment
  axis are company-defined (Linear document "Segment Data for the Backfill and the Incremental
  Update: Findings", 2026-10-18).
- A segment reports a few lines. The profit of a segment is the measure that the chief operating
  decision maker uses (ASC 280-10-50-27), not always operating income under US GAAP.
- A segment value often has a second axis, such as segment × product.

Terms in this ADR:

- **Segment axes:** the axes that the plug-in of a concept names through the trait `SegmentAxes`.
  For `us_gaap`, the seven axes of `xbrl/DESIGN.md`, "Segment Axes".
- **Segment fact:** a fact with dimensions whose axes are all segment axes.
- **The tree:** the roll-up tree, its closed-group rule, its checks, and its other lines, with the
  balance sheet identity, the period identity `core/period/q4-from-fy@1`, and the sums of Tier 4.

The decision binds `core::taxonomy`, `core::alias`, `core::closed_group`, `core::roll_up_check`,
`core::period_identity`, `core::chronology`, the Tier 4 engine in `core::graph`, each taxonomy
plug-in, the silver writer, and the rows of `candidates` and `standardized_facts`.

## Decision

1. **A segment fact gives a candidate under its own dimensions.** Tiers 1 and 2 run on it with the
   alias lists of the consolidated value. The candidate keeps the pairs as filed, with no mapping
   and no change of form. A fact with any other axis gives no candidate. The plug-in that resolves
   the concept names its segment axes through a new trait `SegmentAxes`, next to `Taxonomy`. `core`
   reads only the kind of each axis, so it names no axis of a taxonomy.
2. **The tree and chronology run only on values without dimensions.** A segment value gets no
   derived or computed value, no check, no other line, no Q4 from the fiscal year, and no chronology finding.
   It still gets `restated`.
3. **A segment value is a row of `standardized_facts`.** The table and its key stay as they are.
   The key holds `dimensions` already, and `dimensions` is empty for a consolidated value.

## Options Considered

Measured on 2026-11-15 on `2026_10_notes.zip` of the SEC, 978 10-K and 10-Q filings, with the
scripts listed under "Sources".

| Option | Source-agnostic | New columns | Tree on segment values |
| ------ | --------------- | ----------- | ---------------------- |
| **A: standardized facts with dimensions as filed, no tree (chosen)** | Yes, the same pairs from Notes and the instance | None | No |
| B: A, with the tree on each set of dimensions | Yes | None | Yes |
| C: a separate dataset of segment values | Yes | A dataset and its writer | No |
| D: segment values only in the as-is layer | Yes | None | No |

- A uses the key and the code paths that exist. `SlotKey` of `core::roll_up_check` and the key of
  `core/dedup/newest-filing@1` hold the dimensions already.
- B applies identities that do not hold for segments. Operating income fails to add up over the
  segments in 458 of 851 groups (54%), and a segment reports a few lines, so most closed groups of
  a segment miss a member. A derived Q4 mixes the 10-K and the 10-Q, two reports. The lead decided
  to show segments per report.
- C splits one fact type in two. A reader then joins two datasets for one concept, and the writer
  needs a second path.
- D leaves the alias lists out. A reader then maps `Revenues` and
  `RevenueFromContractWithCustomerExcludingAssessedTax` again for segments. 371 and 218 filings use
  these two on the business-segment axis.

Where the segment axes live, checked against `xbrl/DESIGN.md`, "The `Taxonomy` Trait", on
2026-11-16:

| Option | An IFRS filing | Change to `Taxonomy` |
| ------ | -------------- | -------------------- |
| **A: a trait `SegmentAxes` in each plug-in (chosen)** | Gets segment values once `ifrs` implements the trait | None |
| B: a `const` list in `core` | Gets none. The list names US GAAP and `srt` axes only | None |
| C: a method on `Taxonomy` | Gets segment values | Every plug-in implements it |

- B makes `core` name the axes of one taxonomy, while the alias lists already live in the plug-in.
- C changes a trait that other code implements (`AGENTS.md`, "Open-Closed Principle"). A plug-in
  with no segment data then implements an empty method.

Pairs filed in two forms stay apart: of 1,750 segment values that a filing reports both with
`BusinessSegments` alone and with `ConsolidationItems` = `OperatingSegments` added, 647 differ. So
a rule that merges the two forms loses values.

## Consequences

- `candidates` gains rows. A filing has a mean of 106 values without dimensions and 79 segment
  values on a concept of an alias list (median 110 and 52, maximum 708).
- A read of the consolidated values filters on empty `dimensions`.
- A conflict on a segment fact blocks the candidate of its own dimensions. The consolidated
  candidate stays.
- Members stay company-defined. Two companies compare on a segment only if both use a standard
  member, such as `US` from the `country` namespace.
- Silver stores no label of a member. The company labels of `lab.xml` stay out of the MVP, so a
  reader shows the member name, such as `AmazonWebServices`.
- A company that reports segments only on an axis of its own gets no segment values.
- The decision does not cover the reconciliation. ADR 0030 holds it.

## Open Questions

- The labels of the members: a later design of company labels decides where they live.
- A link between renamed members: the findings of 2026-10-18 leave it out of the MVP.
- The owner of `sec/design/data_model/hybrid_data_model.md` confirms that section 3.4, "Dimensions",
  fits this storage. If not, decision 3 reopens.

## Sources

- `xbrl/DESIGN.md`, "Segment Values", "Dedup Rules", and "Roll-Up Tree", on `main` at `dee3756`.
- `sec/design/pipeline/silver_layer.md`, `standardized_facts`, on `main` at `dee3756`.
- Linear document "Segment Data for the Backfill and the Incremental Update: Findings",
  2026-10-18: the axes, the member shares, and the lead decision on per-report display.
- `measure_segments.py`, `segment_tags.py`, and `segment_volume.py` on
  `https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2026_10_notes.zip`,
  run on 2026-11-15. The scripts and their output are attached to the ticket.
- ASC 280-10-50-27: the measure of a segment item is the measure reported to the chief operating
  decision maker.
