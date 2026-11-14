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
"Layers", calls gold "the serving model of the screener and the `/v1` API".

The parts of a larger goal sit in separate documents:

- `hybrid_data_model.md` §3: the canonical concepts.
- `hybrid_data_model.md` §5: the knowledge graph of companies, identifiers, and source claims.
- `hybrid_data_model.md` §6: the canonical fact store.
- `hybrid_data_model.md` §8.2: the provenance of each fact.
- `ontology_guardrails.md`: how the graph maps onto outside ontologies.

No document names the goal that joins them. The gold design is open
([ADR 0001](0001-mvp-database.md), Amendment 1, "What Stays Open"). It must know what gold is for
before it picks tables and what to copy from silver.

Terms:

- **Metadata store**: the knowledge graph of companies, identifiers, and concepts, with the
  provenance of each fact.
- **Source stores**: the stores that hold the values. These are the fact tier, silver, and the raw
  store.
- **Company brain**: the metadata store over the source stores, in gold.

## Decision

1. **Gold is the company brain.** It holds a metadata store over the source stores. The screener
   and the `/v1` API read gold, and they are two uses of it.
2. **The metadata store holds the knowledge graph, the canonical concepts, and the provenance.**
   The graph holds companies, their identifiers, and the claims that link them
   (`hybrid_data_model.md` §5). Each concept follows §3. Each fact carries the provenance of §8.2.
3. **Gold holds the fact tier and links to silver and the raw store.** A link names a silver unit or
   a raw document by its key. The gold design decides which columns gold copies from silver.
4. **A query of gold can start from the metadata and reach the values.** It starts from a company,
   an identifier, or a concept, and reaches each fact and the source of that fact.

## Options Considered

No option was measured.

| Option | Serves the screener and `/v1` | Answers "where does this value come from" in gold | Databases |
| ------ | ----------------------------- | ------------------------------------------------- | --------- |
| **A (chosen):** gold is the company brain | Yes | Yes | One Postgres (ADR 0001) |
| B: gold is a serving store only | Yes | No, a reader goes to silver | One Postgres |

- A gives one place for the graph, the concepts, the facts, and their provenance, inside the one
  database of ADR 0001.
- B keeps provenance and the graph out of gold, so no query of gold answers the question of the
  owner's goal.

## Consequences

- The gold design starts from the metadata store, not from the screener queries alone.
- Gold grows with provenance. A numeric row with text provenance took about 418 bytes in Postgres
  (ADR 0001, Amendment 1). A link per fact, in place of a copy of each provenance field, keeps the
  row smaller.
- If a trigger of ADR 0001 moves the graph to Neo4j, the metadata store spans two engines. This ADR
  still holds.
- The decision covers gold only. Silver and the raw store keep their designs.

## Open Questions

- The gold tables and their write rule. The gold design fixes them.
- Which provenance fields gold copies from silver, and which it reaches by a link. The gold design
  decides.

## Sources

- [STA-512](https://linear.app/state-machine/issue/STA-512), "Definition of Done": the owner's goal
  and the date, 2026-11-14.
- `sec/design/pipeline/silver_layer.md`, "Layers", on `main` at `070b916`, read on 2026-11-14.
- `sec/design/data_model/hybrid_data_model.md` §3, §5, §6, §8.2, on `main` at `070b916`.
- [ADR 0001](0001-mvp-database.md), Amendment 1: the provenance size of 418 bytes per row,
  measured on 2026-11-08.
