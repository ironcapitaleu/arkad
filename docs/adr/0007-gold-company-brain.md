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

Snowflake describes the same split for its planned Cortex Sense (blog post of 2026-06-30). A
context layer ingests "only metadata and usage patterns, not your actual data rows". It holds the
definitions of metrics, the join patterns, and where each table lives, so an agent "knows exactly
where to look and what to query". When two signals conflict, it ranks them "by relevance,
authority, popularity and freshness", and a conflict it cannot settle goes to a person.
Snowflake reports that an agent with the context layer answered 86.3% of its benchmark questions,
and 24.1% without it. These are vendor figures on Snowflake's data. Nobody checked them for arkad.

Terms:

- **Data source**: an outside publisher of company data, such as the SEC or GLEIF.
- **Source store**: the store of one data source. It holds the facts of that source.
- **Metadata store**: the store that describes the companies, the concepts, and the source stores.
- **Company brain**: the metadata store over the source stores, in gold.

## Decision

1. **Gold is the company brain: one metadata store over one source store per data source.** The
   screener and the `/v1` API read gold, and they are two uses of it.
2. **The metadata store describes the data and holds no fact values.** It holds the knowledge graph
   of companies and their identifiers (§5) and the canonical concepts (§3), each with its
   definition and how to compute it from source facts. For each source store, it records which
   companies, concepts, and periods the store holds, and how to read them. A claim of the graph,
   such as a GLEIF parent link (§5.2), is metadata. It sits in the metadata store with its source
   and date. Only the load writes the metadata store. Unlike the Snowflake context layer, it does
   not learn from usage, such as the queries that reach the `/v1` API.
3. **Each source store holds the facts of its data source, with the provenance of each fact.** The
   SEC store holds the SEC facts, and an ESEF store holds the ESEF facts. The provenance of a fact
   (§8.2) stays next to the fact and names its silver unit and raw document by key. This decision
   covers the facts as each source reports them. Where the canonical facts live is open (Open
   Questions).
4. **A query starts in the metadata store.** It resolves a company, an identifier, or a concept,
   and finds the stores that hold the facts.
5. **The metadata store keeps conflicting claims.** Each claim keeps its source and date. ADR 0001,
   "Design Rules" 2 and 3, rank the claims and fill the review queue. The claim resolution spec of
   ADR 0001, "Open Questions", fixes the priority order and the queue.

## Options Considered

No option was measured.

| Option | Serves the screener and `/v1` | A query finds which source holds a fact | Adding a data source |
| ------ | ----------------------------- | --------------------------------------- | -------------------- |
| **A (chosen):** a metadata store over one source store per data source | Yes | Yes, in the metadata store | A new source store and its entries in the metadata store |
| B: a metadata store over one shared canonical fact tier, with provenance per fact | Yes | Yes, in the provenance of each fact | A new adapter and its resolution map. The shared tables do not change |
| C: gold is a serving store of the graph and the facts, with provenance in silver only | Yes | No, a reader goes to silver | A new adapter and its resolution map |

- A keeps the facts of each data source apart, with their provenance, and gives one place that
  says where each fact lives. It is the split of the Snowflake context layer: metadata in one
  place, data rows in their stores.
- B is the hub-and-spoke model of `hybrid_data_model.md` §2. It mixes the facts of all sources in
  one tier, and only the provenance of a fact names its source. A covers the facts as each source
  reports them. For the canonical facts, B stays possible until the first open question is answered.
- C answers the screener queries, but no query of gold answers where a value comes from.

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
- [Introducing Cortex Sense: Grounded Context for the Data You Never Modeled](https://www.snowflake.com/en/blog/enterprise-ai-agents-grounded-context/),
  Snowflake blog, 2026-06-30, read on 2026-11-14. The quotes and the benchmark figures above come
  from it.
- `sec/design/pipeline/silver_layer.md`, "Layers", and `sec/design/data_model/hybrid_data_model.md`
  §3, §5, §6, §7, §8.2, on `main` at `070b916`, read on 2026-11-14.
- [ADR 0001](0001-mvp-database.md), Amendment 1: the provenance size of 418 bytes per row,
  measured on 2026-11-08.
