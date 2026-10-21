# ADR 0001: Physical Store for the MVP

| Field | Value |
| ----- | ----- |
| Status | Proposed |
| Date | 2026-10-21 |
| Deciders | The project lead and the master's student |
| Ticket | [STA-272](https://linear.app/state-machine/issue/STA-272) |
| Closes | The deferred decision in `sec/design/data_model/hybrid_data_model.md` §14, for the MVP |

## Context

`hybrid_data_model.md` §14 deferred the physical store on purpose. The pipeline writes through the
`storage` ports (§14.F), so the store can change without a pipeline change.
[Choosing the First Storage Backend: Findings](https://linear.app/state-machine/document/choosing-the-first-storage-backend-findings-95eae7d5fbc6)
chose a filesystem raw store and deferred the structured store.

The May 2027 MVP needs a structured store by January 2027. The screener and the `/v1` API query
across all companies, and the current ports read one record by key.

Earlier spikes fixed these inputs:

- The `/v1` API returns fundamentals only. Prices live in a separate store for the screener
  ([STA-270](https://linear.app/state-machine/issue/STA-270)).
- Coverage at launch is the S&P 500. The fact store is append-only, and the newest filing wins
  ([STA-271](https://linear.app/state-machine/issue/STA-271),
  [STA-284](https://linear.app/state-machine/issue/STA-284)).
- Every fact keys on a stable internal `company_id`. CIK and LEI are dated identifiers
  ([STA-288](https://linear.app/state-machine/issue/STA-288)).
- Data checks gate publishing, not persisting. Quarantine works per company
  ([STA-289](https://linear.app/state-machine/issue/STA-289)).
- Supabase already runs Postgres for authentication.

Scale today: about 520 companies, 542 CIK range rows, and a few hundred merger, spin-off, and
subsidiary links. Two to three hops cover every known graph query.

This ADR chooses the store. It does not define the table schema. The schema waits for the Transform
stage to define its output.

## Decision

Use **one managed Supabase Postgres database** for the graph tier and the canonical fact tier. This
is Option A of §14. The raw store stays on the filesystem, as the earlier findings decided.

- Do not use `pg_duckdb`. Supabase does not offer it.
- Store the graph as plain tables behind the graph port. Recursive CTEs answer the traversals.
- Add Neo4j when the first trigger in [Triggers to Revisit](#triggers-to-revisit) fires.
- Keep the graph model ready now. [Entity Resolution and the Graph](#entity-resolution-and-the-graph)
  lists the five design rules.

## Query Needs of the MVP

The frontend repository `ironcapitaleu/frontend` defines the query needs. Today the screener reads
a sample list and filters and sorts in the browser. It does not call the backend yet.

| Query | Frontend type | What it returns | Size |
| ----- | ------------- | --------------- | ---- |
| Q1 Screener snapshot | `Stock` | One row per company with the latest values | About 500 rows |
| Q2 Peer quartiles | `SectorBenchmark` | Lower quartile, median, upper quartile, and peer count, per metric and peer group | About 500 values per metric |

The `Stock` fields come from three sources:

| Source | Fields |
| ------ | ------ |
| Reference data | `symbol`, `name`, `sector`, `country` |
| Price store | `price`, `marketCap`, `changePercent1M`, `weekLow52`, `weekHigh52` |
| Fundamentals | `quickRatio`, `currentRatio` |
| Price and fundamentals | `peRatio`, `priceToCash`, `priceToFcf`, `dividendYield`, `buybackYield` |

`SectorBenchmark` covers eight metrics. Operating margin and return on equity need fundamentals
only. Dividend yield, buyback yield, P/E, P/FCF, P/B, and EV/EBIT also need prices. Each quartile
lists the latest 10-K of each peer as its source.

Both queries map to the read side of `hybrid_data_model.md` §12b.5 without a new repository:

- `FactRepository` gets one more method: the latest values of given metrics for all companies.
- A service above the repositories computes the quartiles and the price ratios. It must sit there,
  because the price ratios combine `FactRepository` with the price store.

Both queries are small. Postgres computes the quartiles with `percentile_cont`. Nothing at this
scale needs a column store or a graph engine.

## Options Considered

### Option A: One Postgres Database (Chosen)

| Pros | Cons |
| ---- | ---- |
| One database, one backup, no sync job | Recursive CTEs are longer to write than Cypher |
| The graph tier and the fact tier share one transaction | `pg_duckdb` is not available on Supabase |
| Supabase already runs it for authentication | A move to a graph database or a lakehouse comes later |
| Mature async Rust drivers (`sqlx`, `tokio-postgres`) | One node sets the scaling limit |

### Option B: Separate Stores from Day One

| Pros | Cons |
| ---- | ---- |
| Native graph queries and RDF tooling from day one | A second system and a sync job before the MVP |
| No later move for the graph tier | Reads across two stores do not see one consistent snapshot |
| | The Rust driver `neo4rs` is still `0.9.0-rc.10` and an "experimental Neo4j Labs project" |
| | The `neosemantics` (n10s) plugin lags Neo4j by about 16 months |

### Label Mapping

The lead's input of 2026-10-17 on STA-272 used its own labels. Its option A is "Postgres plus
Neo4j from day one", which is §14 Option B. Its option B is "Postgres now, the graph as tables,
Neo4j on a fixed trigger", which is §14 Option A plus a trigger. This ADR uses the §14 labels.

### Cost Model

A cost model of 2026-10-19 compared hosting and developer time over three years. All its data sizes,
traffic levels, and effort figures are assumptions. The prices below match the official Supabase
pages, read on 2026-10-21.

| Option in the cost model | 3-year hosting | 3-year extra developer time | 3-year total |
| ------------------------ | -------------: | --------------------------: | -----------: |
| Postgres on Supabase | $1,698 | $0 | $1,698 |
| Postgres and a CDN snapshot | $1,136 | $8,800 | $9,936 |
| Postgres and a Parquet export | $1,664 | $10,400 | $12,064 |
| Lambda and DuckDB on Parquet | $1,590 | $31,200 | $32,790 |
| Iceberg on S3 Tables with DuckDB | $1,487 | $37,600 | $39,087 |
| Iceberg on S3 Tables with Athena | $4,937 | $43,200 | $48,137 |

Developer time uses a placeholder rate of $800 a day. Hosting differs by at most about $3,800 over
three years. Developer time decides the total. The cost model is a Linear document, and it is
not in this repository.

## Supabase Extension Check

Checked on the [official extension list](https://supabase.com/docs/guides/database/extensions)
on 2026-10-21:

| Extension | On Supabase | Effect |
| --------- | ----------- | ------ |
| `pg_duckdb` | No | The column-store path of §14.A needs self-hosted Postgres |
| Apache AGE | No | Matches the lead's input to skip AGE |
| DuckDB wrapper | Yes, read only | Reads Parquet, Iceberg, and S3 Tables from Postgres |
| Iceberg wrapper | Yes, append only | Writes rows to Iceberg tables from Postgres |

The two wrappers give a later lakehouse path without leaving Supabase. Sources:
[DuckDB wrapper](https://supabase.com/docs/guides/database/extensions/wrappers/duckdb),
[Iceberg wrapper](https://supabase.com/docs/guides/database/extensions/wrappers/iceberg).

## Entity Resolution and the Graph

Entity resolution needs a graph model, not a graph database yet. A graph database will come later.

### Why It Needs a Graph Model

Identity is a graph. One company holds several identifiers: CIK, LEI, DART corp code, and national
register numbers. Each source claims links between them, for example "this LEI belongs to this CIK".
Resolution follows those links. It groups every identifier that the links connect into one
`company_id`. This grouping is a graph operation called connected components.

### Why Not a Graph Database Yet

- With SEC data only, the CIK is the key. Resolution is a lookup in dated CIK ranges and
  predecessor links.
- With GLEIF or DART, most matches are exact key joins. Postgres does joins well.
- Grouping runs once per load as a batch job, with union-find in Rust or a recursive CTE. It does
  not run per user request.

### Why a Graph Database Will Come Later

Live multi-hop questions over large ownership chains need a graph database. An example is "the
ultimate parent across all sources". RDF and SHACL tooling, such as Neo4j `neosemantics`, also
needs one. The trigger is in [Triggers to Revisit](#triggers-to-revisit).

### Design Rules

1. Identifiers are rows, not columns. Each row holds `company_id`, scheme, value, valid from, valid
   to, and source.
2. A link between identifiers is a claim with a source and a date, not a fact. Conflicting claims
   sit side by side, and the resolver chooses at read time. `ontology_guardrails.md` describes this
   claim envelope.
3. Grouping uses exact keys only. Fuzzy matches by name or address go to a review queue and are
   never chained. Chaining A≈B and B≈C merges A and C by mistake.
4. `company_id` is stable. Nothing reuses or rewrites it. A merge or a split is a dated event
   (`merged_into`), so facts never move.
5. One resolver port hides the engine. The engine can change from SQL to Neo4j without a pipeline
   change.

## RDF, OWL, and SHACL Later

Neither option needs a migration now to add RDF later.

| Route | Option A | Option B |
| ----- | -------- | -------- |
| Export job | Tables to Turtle files, then a SHACL engine validates the files | Not needed |
| Virtual RDF view | Ontop maps Postgres to RDF and translates SPARQL to SQL through R2RML. It adds one Java service. | Not applicable |
| Plugin | Not applicable | n10s imports OWL and RDF, exports RDF, and validates SHACL on self-hosted Neo4j |

The cost of a later export job or an Ontop mapping depends on four choices made now:

- A stable `company_id` forms the end of each company URI.
- Edge names match FIBO and GLEIF terms from day one, for example "directly consolidated by".
- Claims carry a source and a date. They map to RDF named graphs.
- Identifiers are rows. They map to the identifier terms of FIBO.

RDF earns its place when one of these conditions holds:

- The product links to outside ontologies, such as FIBO or GLEIF, for data exchange.
- A customer or a partner asks for RDF or SPARQL.
- Reasoning goes beyond what a recursive CTE does, for example OWL rules on control and ownership.
  A transitive parent chain alone does not need RDF.

### What OWL Needs

OWL reasoning needs four parts:

1. **IRIs.** Every company, identifier, and edge type has a global IRI. The stable `company_id`
   gives the company IRI.
2. **An ontology.** A file defines the classes, the properties, and the axioms, for example "the
   `ultimatelyConsolidatedBy` property is transitive". It reuses FIBO and GLEIF terms and adds own
   terms only where none exist.
3. **Data in RDF.** The export job or the Ontop view gives the data as RDF triples.
4. **A reasoner.** A batch job reads the ontology and the data, and it writes the derived facts.
   The job writes each derived fact back as a claim with the source "reasoner".

OWL reads data under an open world. Missing data is unknown, not an error. Validation needs a closed
world, so validation uses SHACL, not OWL.

The first two parts cost little and can start at any time. The other two parts wait until RDF
earns its place.

## Triggers to Revisit

| Trigger | Action |
| ------- | ------ |
| The first GLEIF parent-chain load or the second regulator, whichever comes first | Add Neo4j behind the graph port. Confirm that an n10s release exists for the chosen Neo4j version. |
| The fact store passes about 50 GB, for example at full US coverage | Add Iceberg next to Postgres, not in place of it |
| Historical screens over many years and thousands of companies become a product goal | Add Iceberg next to Postgres |
| Screener traffic nears 1.5 million loads a month, or database load or egress grows | Serve the screener as a CDN snapshot |
| One of the RDF conditions above holds | Add the RDF export job, or the Ontop view |

## Consequences

- The Load stage implements the `storage` ports against one Postgres database.
- The graph port and the resolver port hide the engine, so Neo4j comes in as a new implementation.
- `pg_duckdb` is out. If Postgres scans become slow, the next steps are the CDN snapshot or the
  Supabase wrappers to Iceberg.
- The team runs no second database before the MVP.
- Graph queries use recursive CTEs until the Neo4j trigger fires.

## Open Questions

- **Raw archive.** The cost model proposes to keep the raw files of each source unchanged in object
  storage from day one. This touches §12b and §14 and is for the lead and the student to decide.
- **Identity in the design documents.** `load_superstate_design.md` §3.2 (`LeiResolver`, "LEI
  preferred") and `hybrid_data_model.md` §4 (`CompanyId::Cik(..)`) treat the LEI or the CIK as the
  identity. STA-288 chose an internal `company_id`. A rename of the port to `EntityResolver`, which
  returns `company_id` for any `(scheme, value)` pair, removes the mismatch. The paths in §12b.5 use
  `{lei}` and need the same change.
- **Base domain for IRIs.** The company IRI needs a domain. The company is not founded yet.
- **Sector source.** The epic fog log holds a first suggestion: SEC SIC codes and an own roll-up
  table.
- **Fact count.** The cost model assumes 15 million fact rows, or about 4.5 GB in Postgres. A count
  against the real SEC company facts for the S&P 500 will confirm the compute size.

## Sources

"Verified" means read on the official page or in the official repository on 2026-10-21.

| Claim | Status | Source |
| ----- | ------ | ------ |
| Supabase extension list, wrappers | Verified | supabase.com docs, linked above |
| Supabase Pro plan $25 a month with a $10 compute credit | Verified | [Supabase pricing](https://supabase.com/pricing) |
| Compute Micro $10, Small $15, Medium $60, Large $110 a month | Verified | [Compute and disk](https://supabase.com/docs/guides/platform/compute-and-disk) |
| 8 GB disk included, then $0.125 per GB | Verified | [Disk size](https://supabase.com/docs/guides/platform/manage-your-usage/disk-size) |
| 250 GB egress included, then $0.09 per GB | Verified | [Egress](https://supabase.com/docs/guides/platform/manage-your-usage/egress) |
| `WITH RECURSIVE`, `percentile_cont` | Verified | [PostgreSQL docs](https://www.postgresql.org/docs/current/) |
| `pg_duckdb` is MIT and offers self-install, Docker, or MotherDuck | Verified | `duckdb/pg_duckdb` README |
| Neo4j is GPLv3 | Verified | `neo4j/neo4j` LICENSE.txt |
| Latest Neo4j 2026.10.0, latest n10s 2025.06.1 | Verified | Git tags of both repositories |
| `neo4rs` is `0.9.0-rc.10`, a Neo4j Labs project | Verified | `neo4j-labs/neo4rs` README and tags |
| FIBO is MIT | Verified | `edmcouncil/fibo` LICENSE |
| Ontop is Apache 2.0 and maps SQL to RDF through R2RML | Verified | `ontop/ontop` README |
| Fact count, data sizes, traffic, and developer effort | Unverified, assumptions | Cost model of 2026-10-19 |
| GLEIF publishes RDF, GLEIF data is CC0 | Unverified | gleif.org is blocked from the container |
| Aura limits plugins, Neo4j Community has only uniqueness constraints | Unverified | neo4j.com is blocked from the container |
