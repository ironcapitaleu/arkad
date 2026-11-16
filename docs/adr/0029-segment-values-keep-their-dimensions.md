# ADR 0029: Segment Values Keep Their Dimensions and Stay Out of the Tree

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-16 ([STA-502 comment `0ac1cf99`](https://linear.app/state-machine/issue/STA-502/design-segments-in-the-standardized-layer#comment-0ac1cf99)). Amendment 1: Accepted by the project lead on 2026-11-16 ([STA-530 comment `04b019ba`](https://linear.app/state-machine/issue/STA-530/feature-give-segment-facts-candidates-outside-the-roll-up-tree#comment-04b019ba)) |
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
  For `us_gaap`, seven axes, written as in `dimensions`: `BusinessSegments`,
  `SegmentReportingInformationBySegment`, `Geographical`, `ProductOrService`, `ConsolidationItems`,
  `Subsegments`, and `SubsegmentsConsolidationItems`.
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

## Amendment 1: A Segment Fact Gives No Candidate of a Disclosure Concept

Date: 2026-11-16.

**Reason.** Decision 1 says that tiers 1 and 2 run on a segment fact with the alias lists of the
consolidated value, with no exception. Some of those lists belong to a disclosure, not to a
statement. A per-share value divides by the shares of the whole company. On a segment member, such
as `EarningsPerShareBasic` with `BusinessSegments` = `Cloud`, it is no value of that segment. The
rule must hold for each plug-in, so a later `ifrs` plug-in follows it too.

**Decision.** A segment fact gives no candidate of a standard concept of kind
`CanonicalElementKind::Disclosure`. It still gives its `Conflict` finding, and it still counts in
the part rule. Every other alias list of decision 1 applies to it as before.

**Options.**

| Option | Per-share value on a segment | A new disclosure kind |
| ------ | ---------------------------- | --------------------- |
| **A: no candidate of any disclosure concept (chosen)** | No candidate | No candidate until an amendment allows it |
| B: no candidate of `PerShare` only | No candidate | Gets segment candidates with no review |
| C: no exception | A candidate that counts the shares of the whole company | Gets segment candidates with no review |

- On 2026-11-16, `DisclosureConcept` in `xbrl/src/core/elements/mod.rs` at `e5eeef8` holds one
  kind, `PerShare`. So A and B give the same candidates today.
- A makes each new disclosure kind a decision. B and C let a new kind reach `candidates` on a
  segment without one.

**Consequences.**

- A new disclosure kind that holds values per segment needs a further amendment.
- The rule covers tiers 1 and 2 only. The as-is layer keeps the per-share fact with its dimensions.

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

Where the segment axes live, checked against the trait `Taxonomy` in
`xbrl/src/core/taxonomy/mod.rs` at `dee3756`, on 2026-11-16:

| Option | An IFRS filing | Change to `Taxonomy` |
| ------ | -------------- | -------------------- |
| **A: a trait `SegmentAxes` in each plug-in (chosen)** | Gets segment values once `ifrs` implements the trait | None |
| B: a `const` list in `core` | Gets none. The list names US GAAP and `srt` axes only | None |
| C: a method on `Taxonomy` | Gets segment values | Every plug-in implements it |

- B makes `core` name the axes of one taxonomy, while the alias lists already live in the plug-in.
- C modifies `Taxonomy`, a trait that other code implements. A default method keeps every plug-in
  compiling, but the project keeps such a trait closed if a new trait can hold the change, and a new
  trait can. With C, a reader of `Taxonomy` also cannot tell which plug-ins have segment data.

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
- The decision does not cover how the segments reconcile with the consolidated value.

## Open Questions

- The labels of the members: a later design of company labels decides where they live.
- A link between renamed members: the findings of 2026-10-18 leave it out of the MVP.
- The owner of the data model thesis confirms that a set of axis and member pairs on each fact,
  empty for a consolidated value, fits the thesis. If not, decision 3 reopens.

## Sources

- The key of `core/dedup/newest-filing@1` in `xbrl/src/core/dedup.rs`, and `SlotKey` in
  `xbrl/src/core/roll_up_check.rs`, at `dee3756`: both hold the dimensions.
- Linear document "Segment Data for the Backfill and the Incremental Update: Findings",
  2026-10-18: the axes, the member shares, and the lead decision on per-report display.
- `measure_segments.py`, `segment_tags.py`, and `segment_volume.py` on
  `https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2026_10_notes.zip`,
  run on 2026-11-15. The scripts and their output are attached to the ticket.
- ASC 280-10-50-27: the measure of a segment item is the measure reported to the chief operating
  decision maker.
