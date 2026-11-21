# ADR 0043: Standardize Filing Runs Once per Source

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project owner confirms |
| Date | 2026-11-21 |
| Deciders | The project owner |
| Ticket | [STA-332](https://linear.app/state-machine/issue/STA-332) |

## Context

The standardize filing step runs steps 1 and 2 of the dedup order of `xbrl` on one filing:
`core/dedup/more-precise@1`, then tiers 1 and 2 of resolution. It writes the `candidates` rows and
the `Conflict` findings of the filing.

One filing can reach silver from two sources: as rows of a Notes file, and as its instance
document. Silver keeps both sets of as-is facts, one per `source_kind`. A read of `as_is_facts`,
`filings`, and `candidates` keeps one unit per filing: the instance over Notes, and the quarterly
Notes file over the monthly one. The silver contract accepts `candidates` per filing or per source,
and leaves the choice to STA-332.

The two sources of one filing reach silver in different units and at different times. The
instance comes in the `filing_day` unit of its filed date. The Notes rows come in the `notes_file`
unit of a later Notes period, at least weeks later.

Measured on 2026-10-31 on one 10-Q in both sources (STA-402): both parsers give the same value
keys, and the same content after the losses of Notes. The instance keeps units, text, and
footnotes that Notes loses.

This choice binds the filing step, the layout of `candidates`, and the series step that reads it.

## Decision

1. **The standardize filing step runs once for each source of a filing.** It reads only the as-is
   facts of that source, in the same filing machine that parsed them.
2. **Each source writes its own `candidates` rows and `Conflict` findings,** in the unit of that
   source. The read rule of the silver contract picks one unit per filing for the series step.

## Options Considered

| Option | Waits for the other source | Reads silver during a transform | `candidates` rows of one filing |
| ------ | -------------------------- | ------------------------------- | ------------------------------- |
| **A. Once per source (chosen)** | No | No | One set per source. A read takes one |
| B. Once per filing, on the facts of the source that the read rule prefers | Yes, or runs again when the preferred source arrives | Yes: it must find the facts of the other source | One set |

- A keeps the filing step free of I/O, so a unit depends only on its own raw documents.
- B adds a read of `as_is_facts` to every filing, and a second run when the instance arrives after
  the Notes file. It gives the series step nothing that the read rule does not already give.

## Consequences

- A filing in both sources has two sets of `candidates` rows. The read rule of the silver contract
  takes the instance set. So the series step sees one candidate per filing and standard concept.
- The `candidates` of the Notes set cost storage that a default read never takes. The size of one
  filing in `candidates` is about 130 rows (estimate).
- A rule change of standardize filing rewrites the units of both sources in a replay.
- Not covered: a filing whose two sources give different values. Silver keeps both. The parity
  test of the parsers covers the difference.

## Open Questions

- If a later source of the same filing, such as ESEF, needs facts of two sources at once, this
  decision reopens.

## Sources

- `xbrl/src/core/dedup.rs`, `xbrl/src/core/alias.rs`, and `xbrl/src/core/cadence_filter.rs` on
  `main` at `7e2c83e`: the functions of standardize filing, each on the facts of one filing.
- [Linear document "STA-425 Findings: Data Model and Processing of the Silver Layer"](https://linear.app/state-machine/document/sta-425-findings-data-model-and-processing-of-the-silver-layer-2cfdecb2c70a),
  section 6, 2026-11-07: silver keeps both sources of one filing.
- [STA-402](https://linear.app/state-machine/issue/STA-402), 2026-10-31: the comparison of both
  sources on one filing.
