# Storage Abstraction — Proposed Trait Design (WORKING DRAFT)

> Context: STA-130 SPIKE, Option A chosen (Postgres-centric, migration-gated split —
> see `sec/design/data_model/hybrid_data_model.md` §14.A). This captures the DI/trait design
> converged on in discussion, committed to the feature branch for cross-device handoff and
> red-teaming. It is summarized in the SPIKE doc **§14.F**, but **not yet reflected in Linear** —
> still a draft to iterate on before cutting FEATURE tickets.
>
> **Design converged (this revision):** composition-over-inheritance via associated types (the
> `SecClient` pattern), per-store associated `type Error` bounded to convert into one `StorageError`
> currency, concrete/static wiring with **no `dyn`**. See "Revision note" at the bottom for what
> changed from the first draft and why.

## Goal

Abstract away the physical storage so the pipeline never depends on Postgres/Iceberg/graph-DB.
Same pattern as `SecClient`: define traits → a real struct implements them → fakes implement
them for tests. Swapping the physical store later (Option A → B, or a tier → Iceberg) is a new
`impl` behind the same trait, not a pipeline rewrite (`hybrid_data_model.md` §14.D.3, §14.E).

## Design pattern — Repository (decoupling from the storage backend)

The storage layer applies the **Repository pattern**: the pipeline depends on an abstract,
domain-typed persistence interface, never on a concrete database. Swapping the physical backend
(Postgres → Iceberg → a graph DB, or any mixture) is a new implementation behind the same
interface, not a pipeline change. This is the same ports-and-adapters shape the codebase already
uses for `SecClient` (abstract trait → real impl → fake).

**How we implement it:**

1. **The abstraction (the "repository").** A composing `Repository` trait exposes what the pipeline
   needs — `ingest(IngestionUnit)` for writes, domain-typed queries (`query`, `completeness`,
   `ownership_tree`) for reads — in domain types only (no rows/SQL/Cypher). It *has-a* store per
   tier via associated types (`type Raw: RawStore`, `type Graph: GraphStore`,
   `type Facts: FactStore`), each itself a small persistence port.
2. **The implementations (the "adapters").** Concrete backends implement the traits:
   `PostgresRepository` (all three tiers on one pool), or a mixed deployment (data-lake raw +
   graph-DB + Iceberg facts). The pipeline is blind to which.
3. **Test doubles.** In-memory fakes implement the same traits, so pipeline/state tests run with
   zero database.
4. **Decoupling guaranteed by the crate boundary.** Traits + domain types live in the
   storage-agnostic `domain` crate (no `sqlx`); backends live in separate crates. Only the
   composition root ever names a concrete DB.

**Precise note (so the doc doesn't overclaim).** We use "Repository" in the pragmatic
*decouple-from-persistence* sense. Strictly, the composing `Repository` is a persistence **facade**
whose `ingest` carries a **Unit-of-Work** flavor (one atomic multi-tier write); the *read* methods
are classic repository queries; the per-tier `RawStore` / `GraphStore` / `FactStore` are persistence
**ports** (DAO-like), not single-aggregate repositories. The decoupling intent — the reason we
reach for the pattern — is fully honored either way.

> **Background.** For a primer on these patterns (Repository, Ports & Adapters, Unit of Work, CQRS,
> Specification, …) and how arkad combines them, see `design_patterns_primer.md` and the runnable
> `design_patterns_demo.py`.

## Design principles (the load-bearing decisions)

1. **Composition over inheritance — the `SecClient` shape.** A composing `Repository` trait *has-a*
   raw/graph/facts store via **associated types** (`type Raw: RawStore`, …) + accessors — exactly as
   `SecClient` has `type Inner: InnerClient` + `inner()`. **Not** trait inheritance
   (`Repository: RawStore + GraphStore + FactStore`): that would force one type to *be* all three,
   mandating co-location. Has-a lets each tier be a genuinely different engine (see §mixture).
2. **Split by data kind, not one god-trait.** Small per-kind stores/fakes; each test injects only
   the seam it exercises.
3. **Traits speak domain types, never rows/SQL/Cypher/transactions.** Unlike `SecClient`'s
   associated `Request`/`Response` (which abstract the *library* — reqwest's types ≠ a fake's), the
   storage data types (`RawFiling`, `FactSet`, `GraphDelta`, `IngestionUnit`) are the **same across
   every backend** — that is the whole point of the universal model. So capability traits take
   **concrete** domain types; the *only* associated type on them is `Error`.
4. **One error currency, rich impl errors underneath.** Each store has its own associated
   `type Error` (a Postgres store can surface constraint / sqlstate detail; a fake can use a trivial
   error), bounded so it converts into a single concrete `StorageError`. The bound is written
   `StorageError: From<Self::Error>` (the `From` direction — that is what `?` desugars through) and
   declared **once on the base `Storage`** so it propagates to every capability op and to
   `Repository::ingest` for free. `StorageError` *classifies* (retryable / conflict / not-found)
   while preserving the original via `source()`.
5. **Two write paths, on purpose:**
   - `Repository::ingest(unit)` — the **live** path. The *only* write the ETL pipeline calls, so it
     never sequences tier-writes or touches a transaction. Impl owns atomicity (co-located Postgres
     = one tx; mixed-engine = raw-first + async projection — see the contract in §ingest).
   - Per-kind `upsert` + `RawStore::scan` — the **replay/backfill** path for migration/rebuild
     (`hybrid_data_model.md` §14.E). No cross-tier atomicity needed; rebuilding a materialization
     from the raw SoT.
6. **Atomicity stays inside `ingest`'s impl** — never a `type Transaction` / `begin()` on any
   trait (would leak into fakes, a future graph DB, and generic code).
7. **No `dyn` — static composition only.** Associated types make these traits non-object-safe, and
   that is *consistent with the whole architecture*: the framework's `State` trait is already
   non-object-safe (associated types + `Clone`/`Ord`/`Hash` supertraits), so every state is wired by
   concrete type / generics, never `dyn`. Storage follows suit — inject the concrete `Repository`,
   exactly as `SecClient` is a concrete field on a state's context.

## The hierarchy (composition, not inheritance)

```
   Repository                         composing facade — owns atomic `ingest`
   ├─ type Raw:   RawStore            ┐
   ├─ type Graph: GraphStore          ├─ each IS-A Storage (base: identity · health · migrate · Error)
   └─ type Facts: FactStore           ┘
        (accessors: raw() / graph() / facts(), mirroring SecClient::inner())
```

`Repository` *has* three stores; it does not *inherit* their methods. Reach a tier via its accessor:
`store.graph().ownership_tree(root)`, `store.raw().scan(filter)` — reading like `client.inner()`.

## Traits

> **Method signatures below are illustrative, not frozen.** What is settled is the *structure* —
> composition via associated types, the `type Error` → `StorageError` currency, no `dyn`, the crate
> split. The exact method inventory per trait (and whether lifecycle methods like `health()` /
> `migrate()` belong on `Storage` at all) is deliberately deferred until a real consumer forces the
> shape — see Open questions. Read the code below for the *shape of the seams*, not as a decided API.

```rust
use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::error::StorageError; // the ONE currency error (an enum with classification + source)

/// Backend identity for logs/metrics/diagnostics.
pub enum BackendKind { Postgres, Iceberg, Graph, DataLake, Fake }

// ---- Base: universal to ANY store, whatever the data kind ----
// The `where StorageError: From<Self::Error>` lives HERE so every capability op and
// `Repository::ingest` gets the `?`-conversion into the currency for free.
#[async_trait]
pub trait Storage: Send + Sync
where
    StorageError: From<Self::Error>,
{
    /// Impl-specific, rich error. Bounded above to convert into `StorageError`.
    type Error;

    fn backend(&self) -> BackendKind;
    async fn health(&self)  -> Result<(), Self::Error>;
    async fn migrate(&self) -> Result<(), Self::Error>;
    // future cross-cutting hooks live HERE, not on the kinds: backup(), close(), metrics.
    // NEVER put data ops or a transaction/unit-of-work seam on the base.
}

// ---- Data-kind traits: each IS-A Storage. Concrete domain types; error is Self::Error. ----

/// Verbatim filings — durable system of record (§8). Append + replay.
#[async_trait]
pub trait RawStore: Storage {
    async fn append(&self, raw: &RawFiling) -> Result<(), Self::Error>;
    /// Replay source for rebuilding any materialization (§14.E).
    fn scan(&self, filter: RawScan) -> BoxStream<'_, Result<RawFiling, Self::Error>>;
}

/// Knowledge graph — nodes/edges + the two §5.4 read shapes.
#[async_trait]
pub trait GraphStore: Storage {
    async fn upsert(&self, delta: &GraphDelta) -> Result<(), Self::Error>;
    async fn completeness(&self, c: &CompanyId, fy: FiscalYear)   // 1-hop anti-join (§5.4a)
        -> Result<CompletenessReport, Self::Error>;
    async fn ownership_tree(&self, root: &CompanyId)              // multi-hop traversal (§5.4b)
        -> Result<OwnershipTree, Self::Error>;
}

/// Canonical facts — the transactional/analytical tier (§6). May later split behind the
/// scenes into a write-optimized landing store + a read-optimized scan store (Iceberg,
/// compacted on a schedule — §12b.3); callers never learn there are two engines.
#[async_trait]
pub trait FactStore: Storage {
    async fn upsert(&self, facts: &FactSet) -> Result<(), Self::Error>;
    async fn query(&self, c: &CompanyId, q: FactQuery)
        -> Result<Vec<CanonicalFact>, Self::Error>;
}

// ---- Composing trait: HAS-A each store via associated types; owns the atomic write. ----
// `ingest` returns the CURRENCY error (the convergence point), not a per-store Self::Error.
#[async_trait]
pub trait Repository: Send + Sync {
    type Raw:   RawStore;
    type Graph: GraphStore;
    type Facts: FactStore;

    fn raw(&self)   -> &Self::Raw;    // mirrors SecClient::inner()
    fn graph(&self) -> &Self::Graph;
    fn facts(&self) -> &Self::Facts;

    /// The live write path. One filing in, persisted as a unit. Impl owns atomicity.
    async fn ingest(&self, unit: IngestionUnit) -> Result<(), StorageError>;
}
```

A default-ish `ingest` body composes the tiers, and every `?` converts via the base bound:

```rust
async fn ingest(&self, unit: IngestionUnit) -> Result<(), StorageError> {
    self.raw().append(&unit.raw).await?;      // <Self::Raw as Storage>::Error   -> StorageError
    self.graph().upsert(&unit.graph).await?;  // <Self::Graph as Storage>::Error -> StorageError
    self.facts().upsert(&unit.facts).await?;  // <Self::Facts as Storage>::Error -> StorageError
    Ok(())
}
```

> Note the return-type split: **capability ops return the rich `Self::Error`** (caller gets native
> detail, or converts to the currency at will); **`Repository::ingest` returns `StorageError`**
> because it unifies three different store errors and is where cross-cutting logic (retry, logging)
> reads classification.

## `StorageError` — the currency

A concrete enum that **classifies without discarding**: the `From<ImplError>` impl decides the
variant; the original backend error rides along as `source()`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("storage unavailable (retryable)")]
    Unavailable(#[source] Box<dyn std::error::Error + Send + Sync>), // transient — retry
    #[error("conflict / already exists")]
    Conflict(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("not found")]
    NotFound,
    #[error("data integrity / invariant violated")]
    Integrity(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("backend error")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),      // uncategorized
}

impl StorageError {
    /// The one decision the ETL loop needs: retry vs dead-letter.
    #[must_use]
    pub const fn is_retryable(&self) -> bool { matches!(self, Self::Unavailable(_)) }
}
```

Each backend impls `From<ItsError> for StorageError`, mapping (e.g.) a Postgres serialization
failure → `Unavailable`, a unique-violation → `Conflict`, a check-constraint → `Integrity`. Fakes
take the trivial route: `type Error = StorageError` (the reflexive `From<StorageError>` satisfies the
bound with zero boilerplate).

## Mixture of engines — the general case, not a special case {#mixture}

Because `Repository` composes three *independent* associated types, each tier can be a different
engine, and any mixture is expressible:

```rust
// Day-one hybrid (Option B): three engines behind one Repository.
type Raw   = DataLakeRawStore;   // Parquet in object storage
type Graph = Neo4jGraphStore;    // native graph engine
type Facts = IcebergFactStore;   // columnar lakehouse

// Postgres-for-all (Option A) is just the case where all three point at one pool:
type Raw = PostgresRawStore; type Graph = PostgresGraphStore; type Facts = PostgresFactStore;
```

### `ingest` contract — pinned to the weakest guarantee all impls can meet {#ingest}

There are two ways to build the composing repository, and they give **different guarantees**:

- **Co-located native impl** (all three share one Postgres pool) — `ingest` wraps all three writes
  in **one tx**: atomic, read-after-write consistent. The *strong* contract.
- **Generic mixed-engine composition** (three different engines) — **cannot** span one tx, so
  `ingest` is raw-first + async projection: raw durable immediately, graph/facts converge but lag.
  The *weak* contract.

Both satisfy the same `Repository` trait, so the contract is pinned to the weak guarantee:
**on `Ok`, `raw` is durably the system of record; graph/fact materializations converge but may
lag; callers MUST NOT assume read-after-write on the graph/fact tiers.** Option A simply
over-delivers. This is *why* fakes assert on the recorded `IngestionUnit` rather than round-tripping
through `query` — that keeps tests backend-honest instead of silently depending on synchronous
visibility that only Postgres provides.

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

A concrete `PostgresRepository` sets `type Raw/Graph/Facts` to Postgres-backed stores sharing one
pool, and `ingest` opens one tx over all three (the strong contract; the doc's "dual-write is a
non-issue" win, §14.A). A mixed deployment composes different-engine stores and gets the weak
contract — same trait either way.

Fakes are small, per-kind, and compose the same way:

```rust
#[derive(Default)]
pub struct FakeRawStore   { appended: Mutex<Vec<RawFiling>> }
#[derive(Default)]
pub struct FakeGraphStore { deltas:   Mutex<Vec<GraphDelta>> }
#[derive(Default)]
pub struct FakeFactStore  { facts:    Mutex<HashMap<CompanyId, Vec<CanonicalFact>>> }

#[async_trait]
impl Storage for FakeRawStore {
    type Error = StorageError;                 // reflexive From — zero boilerplate
    fn backend(&self) -> BackendKind { BackendKind::Fake }
    async fn health(&self)  -> Result<(), Self::Error> { Ok(()) }
    async fn migrate(&self) -> Result<(), Self::Error> { Ok(()) }
}
// ... impl RawStore for FakeRawStore, etc.

#[derive(Default)]
pub struct FakeRepository {
    raw: FakeRawStore, graph: FakeGraphStore, facts: FakeFactStore,
    ingested: Mutex<Vec<IngestionUnit>>,       // assert what the pipeline handed us
}
#[async_trait]
impl Repository for FakeRepository {
    type Raw = FakeRawStore; type Graph = FakeGraphStore; type Facts = FakeFactStore;
    fn raw(&self) -> &Self::Raw { &self.raw }
    fn graph(&self) -> &Self::Graph { &self.graph }
    fn facts(&self) -> &Self::Facts { &self.facts }
    async fn ingest(&self, unit: IngestionUnit) -> Result<(), StorageError> {
        self.ingested.lock().unwrap().push(unit); Ok(())
    }
}
```

**Fake location (grounded decision).** The `sec` crate's `tests/fixtures` are `#[cfg(test)]` and
crate-internal, so they cannot cross a crate boundary. Because the traits live in the new `domain`
crate (below) but the consuming `Load` state lives in `sec`, the fakes must be reachable from both →
ship them **feature-gated in `domain`** (`#[cfg(feature = "fakes")]`), pulled into `sec`'s dev-deps.
This is the one place we deviate from the skill's "fakes in `tests/fixtures`" convention, and the
deviation is *forced by the crate split*, not preference.

## Crate topology (from the `domain`-crate decision)

The universal core must not depend on any backend, or "storage-agnostic" is a lie the compiler
won't catch. So the decision implies a small topology, not one crate:

```
domain            core types + storage traits + StorageError + feature-gated fakes.
  ▲  ▲  ▲          deps: async-trait, futures, thiserror.  NO sqlx.
  │  │  └── xbrl
  │  └───── sec            (SEC adapter → IngestionUnit; the Load state holds the concrete store)
  │         state_machine (framework)
  │
storage-postgres  PostgresRepository + the three Postgres stores. deps: domain + sqlx.
  ▲
  └── the binary / composition root wires PostgresRepository and injects it
```

Only `main` ever names Postgres; `sec` / `state_machine` / `xbrl` see nothing but `domain` traits.
This only holds if the Postgres impl is a *separate crate* — otherwise `sec`'s dep graph pulls in
sqlx transitively and the boundary rots.

## DI wiring (same as SecClient — concrete, no `dyn`)

Inject the concrete `Repository` type into the `Load` state's **context**, exactly as `SecClient` is
a concrete field on `ExtractSuperStateContext`. Production wires `PostgresRepository`; tests wire
`FakeRepository`. No `dyn`, no trait objects — consistent with the non-object-safe `State` trait.
`Load` calls `store.ingest(unit)`. `CreateFinancialStatements` changes from emitting the placeholder
output to producing `FactSet`s that feed the `IngestionUnit` — the "retire `sec::CompanyData`" work.

> **See also** `load_superstate_design.md` — the Load `SuperState` designed against these ports
> (sub-states, the `FinancialStatementRepository` / `LeiResolver` ports, adapters, and UML).

## What must NEVER leak through a trait (review checklist)

| Leak | Why it kills the swap | Instead |
|---|---|---|
| `sqlx::Transaction` / conn type in a signature | binds callers to Postgres | atomicity inside `ingest` |
| SQL/Cypher strings in or out | binds to a query engine | typed `FactQuery` / domain structs |
| a store's `Self::Error` in `Repository::ingest`'s signature | re-fragments the currency | `ingest -> StorageError`; convert at the seam |
| `sqlx::Error` naked (unconverted) escaping a store | forces callers to look like Postgres | rich `type Error` that `From`-converts into `StorageError` |
| store-specific pagination cursors | leaks engine internals | domain-level page token |
| `type Transaction` / `begin()` on `Storage` | every fake + future graph DB must model it | keep base = lifecycle/observability + `Error` only |

## Payoff of the base trait — generic over `impl Storage`

No `dyn`, so fleet bring-up is a generic function called per concrete store rather than a slice walk:

```rust
async fn bring_up<S: Storage>(s: &S) -> Result<(), StorageError> {
    s.migrate().await?;   // Self::Error -> StorageError via the base bound
    s.health().await?;
    Ok(())
}
// Startup: migrate + health-check every tier, whatever engine.
bring_up(repo.raw()).await?;
bring_up(repo.graph()).await?;
bring_up(repo.facts()).await?;
```

Readiness endpoint, metrics/tracing decorator, "which backends are we on" diagnostic — each written
once against `impl Storage`, blind to data kind. A future Iceberg `FactStore` is picked up for free.

## Revised implementation order (collapses handoff steps 1 & 3)

1. **`domain` crate: core types + storage traits + `StorageError` + feature-gated fakes** — the DI
   contract. (Types and traits are one ticket: the types are the traits' vocabulary.)
2. **`storage-postgres` crate: `PostgresRepository`** implementing the traits (`ingest` = one tx).
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

- **Method inventory is provisional (the big one).** The per-trait method set is *not* frozen —
  is `ingest` the right/only live-write entry point? Do `health()` / `migrate()` belong on the base
  `Storage`, or are they composition-root / lifecycle concerns kept off the data traits? Which reads
  (if any) live here now? Settle when ticket #1's contract is actually cut against a real consumer;
  the structural decisions (composition, error currency, no `dyn`, crate split) are what's locked.
- **Unit-of-ingestion grain:** one `IngestionUnit` per filing (assumed). Confirm.
- **Read repositories now or later?** Lean: *declare* the read methods now (they are the trait
  vocabulary and cost nothing to name), but only *implement* what tests exercise plus
  `RawStore::scan` (replay needs it); leave `PostgresRepository`'s graph reads `todo!()` until a
  screener/API consumer exists (YAGNI). Fakes get real read impls.
- **`FactStore` write/read split:** keep as one trait now; note the future Postgres-landing +
  Iceberg-scan split behind it (§12b.3).
- **Restatement invariant:** `FactStore::upsert` is upsert-*by-full-grain* (incl. `source_ref`),
  never "replace the period's value" — a 10-K/A coexists with the original; "which value wins" is a
  read-time selection. State this so no impl collapses the grain and destroys provenance.

## Revision note — what changed from the first draft

- **`Repository` (composition via associated types)** replaces `FinancialDataStore: RawStore +
  GraphStore + FactStore` (inheritance). Reason: has-a makes the engine *mixture* the natural case;
  is-a forced co-location. Matches the `SecClient` / `InnerClient` house pattern.
- **Per-store `type Error`** (rich, impl-specific) bounded `StorageError: From<Self::Error>` on the
  base `Storage`, replacing the single naked `StorageError` everywhere. Keeps the house `type Error`
  pattern *and* uniform retryability; `?` works because the bound is `From`-shaped.
- **`StorageError` carries `source()`** (boxed) and classifies — no backend detail discarded.
- **No `dyn` anywhere** — was `Arc<dyn FinancialDataStore>` injection + `&[&dyn Storage]` bring-up;
  now concrete/generic, consistent with the non-object-safe `State` trait. Fleet bring-up is a
  generic `bring_up<S: Storage>` called per tier.
- **Crate topology + feature-gated fakes** spelled out as the consequence of the `domain`-crate
  decision.
