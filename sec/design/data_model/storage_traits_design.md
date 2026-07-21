# Storage Abstraction — Proposed Trait Design (WORKING DRAFT)

> Context: STA-130 SPIKE, Option A chosen (Postgres-centric, migration-gated split —
> see `sec/design/data_model/hybrid_data_model.md` §14.A). This captures the DI/trait design
> converged on in discussion, committed to the feature branch for cross-device handoff and
> red-teaming. It is **not yet reflected in the SPIKE doc §14 or in Linear** — still a draft
> to iterate on before cutting FEATURE tickets.

## Goal

Abstract away the physical storage so the pipeline never depends on Postgres/Iceberg/graph-DB.
Same pattern as `SecClient`: define traits → a real struct implements them → fakes implement
them for tests. Swapping the physical store later (Option A → B, or a tier → Iceberg) is a new
`impl` behind the same trait, not a pipeline rewrite (`hybrid_data_model.md` §14.D.3, §14.E).

## Design principles (the load-bearing decisions)

1. **Hierarchical traits.** One thin base `Storage`; each data-kind trait requires it
   (`GraphStore: Storage`, etc.). Implement the base once on the backend; every kind gets it.
2. **Split by data kind, not one god-trait.** Small per-kind fakes; each test injects only the
   seam it exercises.
3. **Traits speak domain types, never rows/SQL/Cypher/transactions.** The trait vocabulary
   *is* the §14.D.1 core types — co-design them together, not in separate tickets.
4. **Two write paths, on purpose:**
   - `FinancialDataStore::ingest(unit)` — the **live** path. Atomic. The *only* write the ETL
     pipeline calls, so it never sequences tier-writes or touches a transaction. Impl owns
     atomicity (Option A = one Postgres tx; Option B = raw-first + outbox projection).
   - Per-kind `upsert` + `RawStore::scan` — the **replay/backfill** path for migration/rebuild
     (`hybrid_data_model.md` §14.E). No cross-tier atomicity needed; rebuilding a
     materialization from the raw SoT.
5. **Atomicity stays inside `ingest`'s impl** — never a `type Transaction` / `begin()` on any
   trait (would leak into fakes, a future graph DB, and generic code).

## The hierarchy

```
        Storage        (identity · health · migrate — universal, cross-cutting)
       ╱   │   ╲
 RawStore GraphStore FactStore     (each: Storage)
       ╲   │   ╱
     FinancialDataStore            (: RawStore + GraphStore + FactStore, + atomic ingest)
```

## Traits

```rust
use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::error::StorageError; // one domain error; NOT sqlx::Error

/// Backend identity for logs/metrics/diagnostics.
pub enum BackendKind { Postgres, Iceberg, Graph, Fake }

// ---- Base: universal to ANY store, whatever the data kind ----
#[async_trait]
pub trait Storage: Send + Sync {
    fn backend(&self) -> BackendKind;
    async fn health(&self) -> Result<(), StorageError> { Ok(()) }  // default: fakes get it free
    async fn migrate(&self) -> Result<(), StorageError> { Ok(()) } // real backends override
    // future cross-cutting hooks live HERE, not on the kinds: backup(), close(), metrics.
    // NEVER put data ops or a transaction/unit-of-work seam on the base.
}

// ---- Data-kind traits: each IS-A Storage ----

/// Verbatim filings — durable system of record (§8). Append + replay.
#[async_trait]
pub trait RawStore: Storage {
    async fn append(&self, raw: &RawFiling) -> Result<(), StorageError>;
    /// Replay source for rebuilding any materialization (§14.E).
    fn scan(&self, filter: RawScan) -> BoxStream<'_, Result<RawFiling, StorageError>>;
}

/// Knowledge graph — nodes/edges + the two §5.4 read shapes.
#[async_trait]
pub trait GraphStore: Storage {
    async fn upsert(&self, delta: &GraphDelta) -> Result<(), StorageError>;
    async fn completeness(&self, c: &CompanyId, fy: FiscalYear)   // 1-hop anti-join (§5.4a)
        -> Result<CompletenessReport, StorageError>;
    async fn ownership_tree(&self, root: &CompanyId)             // multi-hop traversal (§5.4b)
        -> Result<OwnershipTree, StorageError>;
}

/// Canonical facts — the transactional/analytical tier (§6). May later split behind the
/// scenes into a write-optimized landing store + a read-optimized scan store (Iceberg,
/// compacted on a schedule — §12b.3); callers never learn there are two engines.
#[async_trait]
pub trait FactStore: Storage {
    async fn upsert(&self, facts: &FactSet) -> Result<(), StorageError>;
    async fn query(&self, c: &CompanyId, q: FactQuery)
        -> Result<Vec<CanonicalFact>, StorageError>;
}

// ---- Composing trait: IS-A of all three (so also IS-A Storage), owns the atomic write ----
#[async_trait]
pub trait FinancialDataStore: RawStore + GraphStore + FactStore {
    /// The live write path. One filing in, persisted as a unit. Impl owns atomicity.
    async fn ingest(&self, unit: IngestionUnit) -> Result<(), StorageError>;
}
```

## Domain types the traits reference (co-design with §14.D.1 core types)

```rust
/// Everything derived from ONE filing, ready to persist as a unit. The pipeline builds this;
/// it never knows where or how it lands.
pub struct IngestionUnit {
    pub raw:   RawFiling,   // verbatim — SoT
    pub graph: GraphDelta,  // company/filing/concept nodes + edges to upsert
    pub facts: FactSet,     // canonical facts at the §6 grain
}

pub struct RawFiling { /* cik, accession, native tag, taxonomy_version, value, unit, period... */ }
pub struct GraphDelta { /* nodes + edges to upsert (idempotent) */ }
pub struct FactSet { /* Vec<CanonicalFact> for one company/filing */ }
pub struct CanonicalFact { /* §6 grain: company_id, canonical_element, value, unit, period... */ }

pub struct CompanyId(/* LEI preferred, CIK fallback */);
pub struct FiscalYear(u16);
pub struct FactQuery { /* element(s), period range, dimension filter, ... */ }
pub struct CompletenessReport { /* missing periods / missing required concepts */ }
pub struct OwnershipTree { /* root + edges */ }
```

## Real backend + fakes

`PostgresBackend` implements `Storage` once, then `RawStore` + `GraphStore` + `FactStore`, then
`FinancialDataStore` (`ingest` = one Postgres tx over all three tier-writes → the doc's
"dual-write is a non-issue" win, §14.A).

Fakes (in-memory, `SecClient`-style) — base is nearly free thanks to the defaults:

```rust
#[derive(Default)]
pub struct FakeFinancialDataStore {
    raw:   Mutex<Vec<RawFiling>>,
    facts: Mutex<HashMap<CompanyId, Vec<CanonicalFact>>>,
    // graph: Mutex<...>,
    ingested: Mutex<Vec<IngestionUnit>>, // assert what the pipeline handed us
}

#[async_trait]
impl Storage for FakeFinancialDataStore {
    fn backend(&self) -> BackendKind { BackendKind::Fake }
    // health()/migrate() use the trait defaults
}
// ... impl RawStore / GraphStore / FactStore / FinancialDataStore over the in-memory maps
```

Unlocks: pipeline/state tests inject `FakeFinancialDataStore` and assert on the recorded
`IngestionUnit` (no DB); read-path tests inject a single fake `FactStore`, hand-seeded.

## DI wiring (same as SecClient)

Inject `dyn FinancialDataStore` (or `Arc<dyn ...>`) into the `Load` state's **context**, exactly
as `SecClient` is injected. `Load` calls `sink.ingest(unit)`. `CreateFinancialStatements` changes
from emitting the placeholder output to producing `FactSet`s that feed the `IngestionUnit` — this
is the "retire `sec::CompanyData`" work.

## What must NEVER leak through a trait (review checklist)

| Leak | Why it kills the swap | Instead |
|---|---|---|
| `sqlx::Transaction` / conn type in a signature | binds callers to Postgres | atomicity inside `ingest` |
| SQL/Cypher strings in or out | binds to a query engine | typed `FactQuery` / domain structs |
| `sqlx::Error` as the error type | forces every store to look like Postgres | one domain `StorageError` |
| store-specific pagination cursors | leaks engine internals | domain-level page token |
| `type Transaction` / `begin()` on `Storage` | every fake + future graph DB must model it | keep base = lifecycle/observability only |

## Payoff of the base trait — generic over `dyn Storage`

```rust
// Startup: migrate + health-check every store, whatever kind, whatever backend.
async fn bring_up(stores: &[&dyn Storage]) -> Result<(), StorageError> {
    for s in stores { s.migrate().await?; s.health().await?; }
    Ok(())
}
```

Readiness endpoint, metrics/tracing decorator, "which backends are we on" diagnostic — written
once against `Storage`, blind to data kind. A future Iceberg `FactStore` is picked up for free.

## Revised implementation order (collapses handoff steps 1 & 3)

1. **Core types + storage traits** (in `shared/`) — the DI contract. Ship with in-memory fakes.
   (Types and traits are one ticket: the types are the traits' vocabulary.)
2. **`PostgresBackend`** implementing the four traits (`ingest` = one tx).
3. **SEC adapter** — raw parse + resolution map + CIK→LEI, producing `IngestionUnit`s.
4. **Wire `CreateFinancialStatements` → `FactSet` → `Load` calls `ingest`**; retire `sec::CompanyData`.

## Deferred — update semantics: bulk vs incremental (orthogonal axis)

Out of scope for the first trait cut; **noted here so it isn't lost.** This is a *second axis*,
orthogonal to the raw/graph/facts split — it crosses all three tiers, it is not another store kind
(cf. `hybrid_data_model.md` §12 Batch vs Incremental).

- **Bulk** — builds **from zero up to whatever exists now** (initial load / backfill / rebuild).
  Assumes no pre-existing state; it *establishes* the baseline. Idempotent by overwrite/recreate;
  can use engine-native fast paths (empty-DB graph import, `COPY`, Iceberg partition overwrite)
  that a one-at-a-time write can't reach.
- **Incremental** — **requires / assumes existing state**; applies a delta onto it and updates.
  Idempotent by key; the steady-state live path.

Maps onto the live-vs-replay split (incremental = live `ingest`; bulk = the §14.E replay/backfill
path). Likely a separate write-facade (`BulkLoad`) later, rather than doubling every capability
method. **To be designed when a backfill/migration consumer actually exists.**

## Open questions to resolve before cutting tickets

- **Unit-of-ingestion grain:** one `IngestionUnit` per filing (assumed). Confirm.
- **Read repositories now or later?** If ETL-only for now (no API/screener consumer yet),
  build only write + `RawStore`/`FactStore` reads used by tests; leave graph reads as stubs
  (YAGNI). Decide before ticket #1's scope is fixed.
- **`FactStore` write/read split:** keep as one trait now; note the future Postgres-landing +
  Iceberg-scan split behind it (§12b.3).
```
