# Storage Overview

This document gives the shape of storage in arkad: the crates, the ports, the raw store, and the
planned Postgres tiers. Other documents hold the detail. This one links them and repeats none.

Gold is the company brain ([ADR 0007](../../../docs/adr/0007-gold-company-brain.md)). One
**metadata store**, a knowledge graph of all data sources, resolves entities. For each company, it lists the sources that hold data about it and
names the preferred one. It holds the knowledge graph of companies and identifiers, the canonical concepts,
and an entry for each source store ([`hybrid_data_model.md`](./hybrid_data_model.md) §3, §5, and
[`ontology_guardrails.md`](./ontology_guardrails.md)). Each **source store** holds the records of one
data source, such as the SEC facts as filed and as canonical facts, with the provenance of each record
(§8.2). The raw store and silver
feed the source stores. The code holds the raw store. The other tiers are in
[Planned](#planned-silver-graph-and-facts).

## Crates

```text
sec              ──▶ storage    storage: the ports, the errors, and FilesystemRepository
storage-postgres ──▶ storage    planned: the Postgres backend
```

An arrow points from a crate to a crate it depends on. `storage` depends on no database driver, and
`sec` is the only crate of the workspace that depends on it.

Only a composition root, the `main` of a binary, builds a concrete backend. Library code takes a
type that implements a port. [ADR 0005](../../../docs/adr/0005-storage-ports-crate.md) records why
the Postgres backend gets a crate of its own.

## Ports

| Trait | Capability | Design |
| ----- | ---------- | ------ |
| `WriteRepository` | Persist a record | [`capability_split_trait_hierarchy.md`](../../../storage/design/data_model/capability_split_trait_hierarchy.md) |
| `ReadRepository` | Read a record by key | same |
| `ReadWriteRepository` | Marker: read and write | same |
| `ListRepository` | List the documents of a store, as metadata | [`repository_listing.md`](../../../storage/design/data_model/repository_listing.md) |
| `ReadListRepository` | Marker: read and list with one key type | same |

A caller names the port for the access it needs, so a write from a read-only caller does not
compile. A read method returns `ReadError`, and a write method returns `WriteError`. The rustdoc of
`storage::error` describes the types, and
[ADR 0006](../../../docs/adr/0006-storage-error-hierarchy.md) records why they mirror the `sec`
error hierarchy.

## Raw Store

The raw store is the system of record. A rebuild of every other tier starts from it
([`hybrid_data_model.md`](./hybrid_data_model.md) §8). `FilesystemRepository` implements it. It
stores each document as a file under a root directory, with a metadata file next to it. The
binaries of `sec` store one `RawDocument` type per source, such as Company Facts and the Notes
files. [`ingestion_paths.md`](../pipeline/ingestion_paths.md#raw-store), "Raw Store", gives the
keys and the layout.

## Planned: Silver, Graph, and Facts

None of this section has code. It follows
[Amendment 1](../../../docs/adr/0001-mvp-database.md#amendment-1-load-reads-silver) of
[ADR 0001](../../../docs/adr/0001-mvp-database.md).

- **Ingest** writes the raw store only.
- **Transform** reads the raw store and writes silver: Parquet units and the `silver_files` list
  ([`silver_layer.md`](../pipeline/silver_layer.md)).
- **Load** reads one silver unit and writes its graph rows, its fact rows, and a ledger row in one
  Postgres transaction. The backend lives in `storage-postgres`. If the ledger holds a unit, a
  rerun skips it.
- The graph tier and the fact tier share one Supabase Postgres database (ADR 0001). The graph tier
  holds the metadata store. The fact tier holds the facts of the source stores. The gold
  design places the records that are not facts, such as the GLEIF LEI records.
- Read methods beyond `get` and `list`, such as the queries of the screener and the API, wait for
  a consumer that needs them.
