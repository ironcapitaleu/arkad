# ADR 0007: Gold Is the Company Brain

| Field | Value |
| ----- | ----- |
| Status | Accepted by the lead on 2026-11-14 ([STA-512 comment](https://linear.app/state-machine/issue/STA-512/docs-replace-the-storage-trait-design-with-an-overview-and-adrs#comment-e112485d)). |
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
- **Source store**: the store of one data source. It holds the records of that source as the
  source publishes them, such as the SEC facts or the GLEIF LEI records.
- **Metadata store**: the one knowledge graph that describes the companies, the concepts, the data
  sources, and the source stores, and routes a caller to the sources.
- **Company brain**: the metadata store over the source stores, in gold.

## Decision

1. **Gold is the company brain: one metadata store for all data sources, over one source store per
   data source.** The metadata store is a knowledge graph. The screener and the `/v1` API read
   gold, and they are two uses of it.
2. **The metadata store has two jobs: entity resolution and routing.** It resolves a name or an
   identifier to one company (§4, §5). For a company and a kind of information, it lists every
   source that holds it, with the key and how to read it, and names the preferred source
   (Decision 5). For example, the financial facts of Apple Inc. come from the SEC store, by CIK.
3. **The metadata store describes the data and holds no fact values.** Its graph holds four things:
   the companies and their identifiers (§5), the canonical concepts (§3), the data sources, and
   their source stores. The data sources are the `DataSource` nodes of §5.1, and a source store is
   a node of its own. Each concept has its definition and how to compute it from source facts. An
   edge from a company to a source store carries the kind of information, the key, and the rank of
   that store for that company (Decision 5). For a store of facts, the metadata store also records
   the concepts and the periods. A claim of the graph, such as a GLEIF parent link (§5.2), is metadata.
   The load derives each claim from the records of the source store of its source, such as the
   GLEIF store. The claim sits in the metadata store with its source and date. Three writes change the
   metadata store: the load, a verdict on an item of the review queue, and a hand edit of the
   ranking, the home market, or the trust of a source (Decision 5). Unlike the
   Snowflake context layer, it does not learn from usage, such as the queries that reach the `/v1`
   API.
4. **Each source store holds the records of its data source, with the provenance of each
   record.** The SEC store holds the SEC facts, as filed and as canonical facts. The GLEIF store
   holds the LEI records as GLEIF publishes them. The provenance of a record (§8.2) stays next to
   it and names its silver unit and raw document by key. No store holds the canonical facts of two
   data sources.
5. **If several source stores hold records of the same kind for a company, the metadata store
   picks one by default, and the caller can name another.** A caller that names a source reads
   that source. This is the read-time policy of §2.1. It covers the records of the source stores,
   such as facts or a legal address. Decision 6 covers the claims of the graph.
   - **The order is per company.** The source of the home market of the company comes first, then
     the more trusted sources. Each company has its own order, not one global list.
   - **Example.** Alibaba lists in Hong Kong and New York. At the SEC it files the annual 20-F of a
     foreign issuer and furnishes its quarterly results on Form 6-K, without the XBRL facts of a
     10-Q, so the SEC store holds no quarterly facts for Alibaba. If a person sets Hong Kong as its
     home market, and a Hong Kong source store exists, the Hong Kong source ranks above the SEC
     for Alibaba. For Apple, the SEC ranks first.
   - **Where it sits.** The home market of each company and the rank of each source store for it sit
     in the graph. The trust of a source sits on the node of the data source. A person edits all
     three in the graph. At first the trust is a judgment, and a source as well known as the SEC
     gets a high trust.
   - **Default rank.** Until a person ranks a store for a company, the store has a default rank
     from the home market and the trust of the source. If a company has no home market yet, the
     default follows the trust alone. A default rank follows each change of the home market or the
     trust. A load never changes a rank that a person set.
6. **The metadata store keeps conflicting claims.** Each claim keeps its source and date. ADR 0001,
   "Design Rules" 2 and 3, settle the links between identifiers. The claim resolution spec of
   ADR 0001, "Open Questions", fixes the priority order, the review queue, and the rule for the
   other claims, such as parent links.

## Options Considered

No option was measured.

| Option | Serves the screener and `/v1` | A query finds which source holds a fact | Adding a data source |
| ------ | ----------------------------- | --------------------------------------- | -------------------- |
| **A (chosen):** one metadata store, a knowledge graph of the companies, the concepts, and the data sources, over one source store per data source, each with its canonical facts | Yes. A screener across regions reads several source stores | Yes, in the metadata store | A new source store and its entries in the metadata store |
| B: a metadata store over one shared canonical fact tier, with provenance per fact | Yes. A screener reads one tier | Yes, in the provenance of each fact | A new adapter and its resolution map. The shared tables do not change |
| C: gold is a serving store of the graph and the facts, with provenance in silver only | Yes | No, a reader goes to silver | A new adapter and its resolution map |

- A keeps the records of each data source apart, with their provenance, and gives one place that
  says where each record lives. It is the split of the Snowflake context layer: metadata in one
  place, data rows in their stores. The metadata store picks the source for each company, so a
  caller does not merge two sources for one company.
- B is the hub-and-spoke model of `hybrid_data_model.md` §2. A screener reads one tier, but the
  tier mixes the canonical facts of all sources, and only the provenance of a fact names its
  source.
- C answers the screener queries, but no query of gold answers where a value comes from.

## Consequences

- The gold design starts from the metadata store and the source stores, not from the screener
  queries alone.
- A new data source adds a source store and its entries in the metadata store. The other source
  stores do not change.
- A screener across regions reads the canonical facts of several source stores. The metadata store
  names the stores and the key for each company.
- Provenance stays with each record. A numeric row with text provenance took about 418 bytes in
  Postgres (ADR 0001, Amendment 1). A key to the silver unit and the raw document, in place of a
  copy of each provenance field, keeps the row smaller.
- `hybrid_data_model.md` §2 (diagram and "Principle"), §2.1, §6, and §7.1 describe one shared
  canonical fact store. They must change to match this ADR.
- Agreement and divergence between two source stores for one company, such as a dual filer at the
  SEC and in ESEF, need a check across stores. In B the shared tier holds both rows side by side.
- The decision names no engine. ADR 0001 picks the engines.

## Open Questions

- A rule that sets the trust of a source from data, in place of a judgment. The error rates and
  conflicts that the data checks find for each source, at ingest and on live data, are the
  candidate input. It needs several sources and those checks first. The owner decides.
- How a hand-set ranking keeps who set it and when, since no pull request reviews it. The gold
  design fixes it.
- Which rule a person follows to set the home market of a company that lists in two markets. The
  candidates are the market of its primary listing and the market of the legal address in its LEI
  record. The owner decides.
- Which source sets the attributes of a `Company` node (§5.1), such as the name and the country,
  when two source stores disagree, and whether the preference order of Decision 5 decides it. The owner decides.
- Whether the claim resolution spec of ADR 0001 ranks the claims of a company by the per-company
  order of Decision 5, or keeps a fixed source priority. The owner and the lead decide.
- Where the check across source stores runs, and where the confidence from agreement across
  sources (§2.1) lives. The gold design fixes it.
- The gold tables and their write rule, and the tier of the records that are not facts, such as
  the GLEIF LEI records. The gold design fixes them.
- How a source store and its entries in the metadata store stay in step on a load. The gold design
  fixes it.

## Sources

- [STA-512](https://linear.app/state-machine/issue/STA-512), "Definition of Done": the owner's goal
  and the date, 2026-11-14.
- The owner's decisions on the draft, 2026-11-14, recorded in a
  [comment on STA-512](https://linear.app/state-machine/issue/STA-512/docs-replace-the-storage-trait-design-with-an-overview-and-adrs#comment-d6de906d): one metadata store, a knowledge graph
  of the companies, the concepts, and the data sources, over one store per data source with its facts and their provenance. The
  canonical facts stay in each source store. The metadata store resolves entities, lists the
  sources of each company with their keys, and picks a source by a hand-set preference order that
  a caller can override.
- [Introducing Cortex Sense: Grounded Context for the Data You Never Modeled](https://www.snowflake.com/en/blog/enterprise-ai-agents-grounded-context/),
  Snowflake blog, 2026-06-30, read on 2026-11-14. The quotes and the benchmark figures above come
  from it.
- `sec/design/pipeline/silver_layer.md`, "Layers", and `sec/design/data_model/hybrid_data_model.md`
  §2, §2.1, §3, §4, §5, §6, §7, §8.2, on `main` at `070b916`, read on 2026-11-14.
- [ADR 0001](0001-mvp-database.md), Amendment 1: the provenance size of 418 bytes per row,
  measured on 2026-11-08.
