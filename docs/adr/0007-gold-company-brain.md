# ADR 0007: Gold Is the Company Brain

| Field | Value |
| ----- | ----- |
| Status | Proposed. The lead confirms. The owner confirms the wording first. |
| Date | 2026-11-14 |
| Deciders | The project lead. The owner sets the goal. |
| Ticket | [STA-512](https://linear.app/state-machine/issue/STA-512) |

## Context

The pipeline has three layers: bronze (the raw store), silver (Parquet), and gold (Supabase
Postgres, [ADR 0001](0001-mvp-database.md)). [`silver_layer.md`](../../sec/design/pipeline/silver_layer.md),
"Layers", calls gold the store that the screener and the `/v1` API read.

The parts of a larger goal sit in separate documents:

- `hybrid_data_model.md` §3: the canonical concepts.
- `hybrid_data_model.md` §5: the knowledge graph of companies, identifiers, and source claims.
- `hybrid_data_model.md` §6: the canonical fact store.
- `hybrid_data_model.md` §7: one adapter per data source, such as the SEC.
- `hybrid_data_model.md` §8.2: the provenance of each fact.
- `ontology_guardrails.md`: how the graph maps onto outside ontologies.

ADR 0001 puts the graph tier and the fact tier in gold. It leaves the gold tables and the way
provenance reaches gold to a later gold design (Amendment 1, "What Stays Open"). No document names
the goal that joins the parts, so that design has no goal to follow.

Terms:

- **Data source**: an outside publisher of company data, such as the SEC or GLEIF.
- **Source store**: the store of one data source. It holds the facts of that source.
- **Metadata store**: the store that describes the companies, the concepts, and the source stores.
- **Company brain**: the metadata store over the source stores, in gold.

## Decision

1. **Gold is the company brain: one metadata store over one source store per data source.** The
   screener and the `/v1` API read gold, and they are two uses of it.
2. **The metadata store describes the data and holds no fact values.** It holds the knowledge graph
   of companies and their identifiers (§5) and the canonical concepts (§3). For each source store,
   it records which companies, concepts, and periods the store holds, and how to read them.
3. **Each source store holds the facts of its data source, with the provenance of each fact.** The
   SEC store holds the SEC facts, and a GLEIF store holds the GLEIF data. The provenance of a fact
   (§8.2) stays next to the fact and names its silver unit and raw document by key.
4. **A query starts in the metadata store and reads the facts from the source stores.** It resolves
   a company, an identifier, or a concept, finds the source stores that hold it, and reads the facts
   there.

## Options Considered

No option was measured.

| Option | Serves the screener and `/v1` | A query finds which source holds a fact | Adding a data source |
| ------ | ----------------------------- | --------------------------------------- | -------------------- |
| **A (chosen):** a metadata store over one source store per data source | Yes | Yes, in the metadata store | A new source store and its entries in the metadata store |
| B: gold is a serving store of the graph and the facts, with provenance in silver only | Yes | No, a reader goes to silver | A change to the shared tables |

- A keeps the facts of each data source apart, with their provenance, and gives one place that
  says where each fact lives.
- B answers the screener queries, but no query of gold answers where a value comes from.

## Consequences

- The gold design starts from the metadata store and the source stores, not from the screener
  queries alone.
- A new data source adds a source store and its entries in the metadata store. The other source
  stores do not change.
- Provenance stays with each fact. A numeric row with text provenance took about 418 bytes in
  Postgres (ADR 0001, Amendment 1). A key to the silver unit and the raw document, in place of a
  copy of each provenance field, keeps the row smaller.
- The decision names no engine. ADR 0001 picks the engines.

## Open Questions

- Where the canonical facts that join data sources live (§6): in each source store, or in a shared
  fact tier that the metadata store describes. The owner decides.
- The gold tables and their write rule. The gold design fixes them.
- How a source store and its entries in the metadata store stay in step on a load. The gold design
  fixes it.

## Sources

- [STA-512](https://linear.app/state-machine/issue/STA-512), "Definition of Done": the owner's goal
  and the date, 2026-11-14.
- The owner's comment on the draft, 2026-11-14: a metadata store with the data endpoints, and one
  store per data source with its facts and their provenance.
- `sec/design/pipeline/silver_layer.md`, "Layers", and `sec/design/data_model/hybrid_data_model.md`
  §3, §5, §6, §7, §8.2, on `main` at `070b916`, read on 2026-11-14.
- [ADR 0001](0001-mvp-database.md), Amendment 1: the provenance size of 418 bytes per row,
  measured on 2026-11-08.
