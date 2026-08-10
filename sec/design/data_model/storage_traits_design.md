# Storage Abstraction — Trait Design

> Context: STA-130 SPIKE. **Decision (2026-08-08): no physical deployment option is chosen at
> this stage — deliberately.** The team decision is to abstract physical storage completely behind
> the `storage` crate's ports (this doc); the physical choice (Option A, B, or any mixture —
> `hybrid_data_model.md` §14) is deferred until a measured trigger forces it.
>
> **Status (2026-08-10): the [Consolidated Design](#consolidated-design-current--2026-08-10) section
> below is current and authoritative** — the shape STA-139 scaffolds. STA-145 froze an earlier v1
> (composing `Repository` with a baked-in `type Raw/Graph/Facts` triad, `ingest`, `StorageError`);
> a subsequent design pass refined it — **neutral `Repository` + `persist`, a `Backend` store base,
> `SecRepository` owning the triad, `ErrorKind` as a value-type hierarchy leaf.** The detailed
> sections after the consolidated view (transaction ownership, mixture-of-engines, the write
> contract) keep their rationale — read `persist` for `ingest`, `Backend` for the base `Storage`,
> and `SecRepository` for the composing `Repository`; the [frozen v1 trait block](#traits-frozen--sta-145)
> is retained for history and marked superseded.

## Consolidated Design (current — 2026-08-10)

The authoritative shape. The sections below this one predate it and are retained for their
rationale (read the mapping in the status note above).

### Naming & structure

| Element | Decision |
| --- | --- |
| **crate** | `storage` — owns the word, so no trait is named `Storage` |
| **facade** | `Repository` — *neutral*: `type Record` + `persist`, backend-blind |
| **write method** | `persist(record)` (was `ingest`) |
| **write-unit** | `Repository::Record` associated type; `sec` binds `type Record = FilingRecord` |
| **store base** | `Backend` — `kind() -> BackendKind` only (identity; errors are operation-classed, lifecycle off-trait) |
| **the three ports** | `RawStore` / `GraphStore` / `FactStore`, each `: Backend` |
| **error** | `error::ErrorKind` — value-type union over `ReadError` / `WriteError` (shared `BackendError`), sec-style `From`/`TryFrom` hierarchy |
| **backends** | `BackendKind::{Postgres, Iceberg, Graph, DataLake, Memory, FileSystem, Fake}` |

### Crate topology

```
xbrl            domain vocab: CompanyId, FactSet, CanonicalFact, FiscalYear
  ▲
storage         PORTS only: Repository · Backend · RawStore/GraphStore/FactStore · ErrorKind
  ▲  ▲            (deps: xbrl, async-trait, futures, thiserror — NO sqlx)
  │  │
  │  storage-postgres   Postgres impls of the 3 ports + PostgresSecRepository (one tx, strong contract)
  │   ▲
  sec             SecRepository<R,G,F>: Repository  (composes the triad; SEC raw schema/mapping;
                  CIK→CompanyId at save; type Record = FilingRecord) + its own #[cfg(test)] fakes
```

Only the composition root ever names Postgres.

### `storage` directory (write-path core)

```
storage/
├── Cargo.toml
└── src/
    ├── lib.rs                 # module decls + re-exports
    ├── repository.rs          # Repository  — neutral facade
    ├── backend.rs             # Backend, BackendKind
    ├── error/                 # sec-style hierarchy (module-per-level)
    │   ├── mod.rs             # ErrorKind (union) + From/TryFrom + DowncastNotPossible
    │   ├── write_error/mod.rs # WriteError
    │   ├── backend_error/mod.rs # BackendError (is_retryable)
    │   └── read_error/mod.rs  # ReadError  (added with the STA-139 read methods)
    └── store/
        ├── mod.rs
        ├── raw.rs             # RawStore   + RawFiling
        ├── graph.rs           # GraphStore + GraphDelta
        └── facts.rs           # FactStore
```

```rust
// repository.rs — the ONLY thing the pipeline injects. Backend-blind.
#[async_trait]
pub trait Repository: Send + Sync {
    type Record;
    async fn persist(&self, record: Self::Record) -> Result<(), WriteError>;   // always a write
}

// backend.rs — what every store shares: identity only (errors are the operation-classed types below).
pub trait Backend: Send + Sync {
    fn kind(&self) -> BackendKind;
}
pub enum BackendKind { Postgres, Iceberg, Graph, DataLake, Memory, FileSystem, Fake }

// store/raw.rs · graph.rs · facts.rs — write-path; each `: Backend`; returns the WriteError class directly
#[async_trait] pub trait RawStore:   Backend { async fn append(&self, filing: &RawFiling) -> Result<(), WriteError>; }        // immutable SoT
#[async_trait] pub trait GraphStore: Backend { async fn upsert(&self, delta: &GraphDelta) -> Result<(), WriteError>; }        // idempotent
#[async_trait] pub trait FactStore:  Backend { async fn upsert(&self, company: &CompanyId, facts: &FactSet) -> Result<(), WriteError>; } // by-full-grain
```

### Error model — mirrors the `sec` error hierarchy (operation-classed)

Built **exactly like `sec/src/lib/error/`**: module-per-level, value-type at every level, `From`
upcast / `TryFrom` downcast, a single `DowncastNotPossible` sentinel on the top, and the same
`implements_*` trait-assertion + cast-round-trip test suite per level. The classes are keyed to the
**kind of operation** — a read method returns a `ReadError`, a write method a `WriteError` — so
illegal states are unrepresentable (`persist` can never hand back `MissingRecord`).

**Layout** (`storage/src/error/`):

```
error/
  mod.rs                → ErrorKind    { Read(ReadError), Write(WriteError), DowncastNotPossible }
  read_error/mod.rs     → ReadError    { MissingRecord, Backend(BackendError), … }   ← arrives with the read methods (STA-139)
  write_error/mod.rs    → WriteError   { ConflictingWrite{reason}, FailedIntegrityCheck{reason}, Backend(BackendError) }
  backend_error/mod.rs  → BackendError { Unavailable{reason}, Failed{reason} }        ← is_retryable() lives here
  // a RICH leaf gets its own module (e.g. missing_read_permission/), exactly as sec splits `invalid_cik_format`
```

**Per-level conventions — identical to sec:**

- `#[non_exhaustive]` + `#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]`. **Value
  type** — no `Box<dyn Error>`; rich detail is flattened to a `reason` string at the conversion
  boundary. This is exactly what lets each level be *contained in* the level above it.
- **Markers stay inline** on the enum with `#[error("[…] …, Reason: '…'")]`; **rich leaves get their
  own module** and are wrapped (`Read(ReadError)`-style). Same split as sec's `State::InvalidInput`
  (inline) vs `State::InvalidCikFormat(_)` (module). Today's causes are all markers → inline.
- **Upcast** = `From<Inner> for Outer` — infallible, `?`-able: `MissingReadPermission → ReadError →
  ErrorKind`. So a read method returns `ReadError` and anything `Into<ReadError>` flows in via `?`.
- **Downcast** = `TryFrom<Outer> for Inner`, `type Error = ErrorKind`, returns
  `ErrorKind::DowncastNotPossible` on mismatch (skip-level allowed, e.g. `TryFrom<ErrorKind> for
  BackendError`). This is the *fallible* direction — not `Into`.

**Method return types — the narrow class, not the union:**

```rust
async fn persist(&self, r: Self::Record) -> Result<(), WriteError>;   // never MissingRecord
async fn query(&self, …)                 -> Result<_,  ReadError>;    // never ConflictingWrite   (STA-139)
```

The store methods return the classes **directly** (`append -> Result<(), WriteError>`); the Postgres
impl maps `sqlx::Error → WriteError::Backend(BackendError::Failed { reason })` at its own boundary.
This **drops the earlier `Backend::type Error` associated error** — the value-type decision already
flattened rich detail to `reason`, so the associated type was buying flexibility we don't use.
`ErrorKind` (the union) is what the **shared consumers** take — the retry decorator (`is_retryable()`)
and the `sec` seam (`State::FailedPersistence(ErrorKind)`) — reached by the `From` upcast.

**Grows with the methods.** Today only write methods exist → build `ErrorKind` + `WriteError` +
`BackendError` now; add `ReadError` (and any rich read leaf) when the read methods land in STA-139.
The union + `#[non_exhaustive]` make that additive, not a rewrite.

### `SecRepository` — where the triad lives

```rust
// sec crate — holds the Backends as swappable generics (the SecClient<Inner> shape).
// Repository and Backend meet ONLY inside persist(); the sole link is the currency bound.
struct SecRepository<R: RawStore, G: GraphStore, F: FactStore> { raw: R, graph: G, facts: F }

impl<R: RawStore, G: GraphStore, F: FactStore> Repository for SecRepository<R, G, F> {
    type Record = FilingRecord;                                   // { company, raw, graph, facts }
    async fn persist(&self, r: FilingRecord) -> Result<(), WriteError> {
        let FilingRecord { company, raw, graph, facts } = r;
        self.raw.append(&raw).await?;                             // each store returns WriteError, so
        self.graph.upsert(&graph).await?;                        // `?` funnels straight through — no
        self.facts.upsert(&company, &facts).await?;              // conversion needed
        Ok(())
    }
}
```

- `Repository` never names `Backend` — they meet only inside `persist`, where the tier stores'
  `WriteError`s funnel through `?` into `persist`'s `WriteError` (no conversion; same class).
- **SEC-specific = the raw tier** (verbatim SEC schema, CIK/accession keys); graph/facts are
  canonical/shared. A future `EsefRepository` is just another `impl Repository` sharing the
  canonical tiers, bringing its own raw store.
- **Strong contract** (one transaction) = concrete `PostgresSecRepository` in `storage-postgres`;
  the generic composed body above is the **weak** contract (raw-first + idempotent projections).
  See [Transaction ownership](#tx-ownership) below — it still applies, with `persist` for `ingest`.

### Fakes — per-crate, house convention

- **Every crate keeps its own `#[cfg(test)]` fakes** under its own `tests/fixtures/`; nothing is
  exported. `storage` → stubs for its own bound-assertions/doctests; `sec` → `FakeRepository` +
  fake stores + sample `FilingRecord`s (orphan rule: local fake type, foreign `storage` trait).
- **Promote trigger:** a *second* consumer needing the *same* fake → extract a `storage-testkit`
  crate (or `#[cfg(feature = "fakes")]`). Not before. (This reverses the older "feature-gated fakes
  in `storage`" note.)

### Deferred to STA-139 (marked, not decided)

1. **Read surface** (`scan` / `completeness` / `ownership_tree` / `query`) + the CQRS read/write split.
2. **`RawStore` genericity** (`type Item`?) — raw is the one regulator-specific tier.
3. **`completeness` placement** — straddles graph structure and fact presence.
4. **Retry / DLQ** — a `RetryingRepository` decorator (reads `is_retryable()`); reconciliation for
   post-raw projection gaps; a DLQ only for poison (non-retryable) records; all at the driver level,
   never in the state machine.

---

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
4. **Decoupling guaranteed by the crate boundary.** The persistence ports live in the
   backend-agnostic `storage` crate (no `sqlx`), which depends on `xbrl` for the domain
   vocabulary; backends live in separate crates. Only the composition root ever names a
   concrete DB.

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
   ├─ type Graph: GraphStore          ├─ each IS-A Storage (base: identity · Error currency)
   └─ type Facts: FactStore           ┘
        (accessors: raw() / graph() / facts(), mirroring SecClient::inner())
```

`Repository` *has* three stores; it does not *inherit* their methods. Reach a tier via its accessor:
`store.graph().ownership_tree(root)`, `store.raw().scan(filter)` — reading like `client.inner()`.

## Traits (FROZEN — STA-145)

> **⚠️ Superseded (2026-08-10) by the [Consolidated Design](#consolidated-design-current--2026-08-10)
> above.** This v1 block is retained for history. What changed: `Repository` lost its baked-in
> `type Raw/Graph/Facts` triad and became a neutral `type Record` + `persist` facade; the triad
> moved to `sec::SecRepository`; the base `Storage` trait was renamed `Backend` (`backend()` →
> `kind()`); `ingest` → `persist`; `IngestionUnit` → `FilingRecord` (in `sec`); `StorageError`
> (boxed-source) → `error::ErrorKind` (Clone/Eq/Ord value type). The **rationale** below —
> transaction ownership, mixture-of-engines, the write contract, the anti-leak checklist — still
> holds; substitute the new names when reading.

> **The signatures below are the frozen contract STA-139 scaffolds verbatim.** Structure
> (composition via associated types, the `type Error` → `StorageError` currency, no `dyn`, the crate
> split) *and* the per-trait method inventory are now settled. The method-inventory question is
> closed as follows:
>
> - **Base `Storage` = `type Error` + `backend()` only.** `health()` / `migrate()` are **not** on the
>   trait — they are composition-root lifecycle concerns called on the concrete type at startup
>   (see [Startup / lifecycle](#lifecycle)). Should a *generic* lifecycle surface ever be needed, add
>   a `Lifecycle: Storage` trait then — not before a consumer exists (YAGNI).
> - **Write path is frozen and live:** `Repository::ingest` (the only path the ETL calls),
>   `RawStore::append` + `RawStore::scan` (replay/rebuild), and the three tier `upsert`s.
> - **Reads (`completeness` / `ownership_tree` / `query`) are frozen *as vocabulary* but implemented
>   lazily** — their return DTOs are named here so the trait shape is stable, but only the fakes get
>   real bodies now; the Postgres impls stay `todo!()` until a screener/API read consumer is cut.
>   The **implementation status** column below is the honest record of what STA-139 fills in vs. what
>   waits.
>
> | Method | Kind | STA-139 implements? |
> | --- | --- | --- |
> | `Repository::ingest` | write (live) | **yes** — the ETL write path |
> | `RawStore::append` / `scan` | write / replay | **yes** — replay feeds rebuilds |
> | `GraphStore::upsert`, `FactStore::upsert` | write | **yes** |
> | `GraphStore::completeness` / `ownership_tree`, `FactStore::query` | read | frozen signature; **fakes only** — Postgres `todo!()` until a read consumer exists |
> | `Storage::backend` | identity | **yes** (trivial) |

```rust
use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::error::StorageError; // the ONE currency error (an enum with classification + source)

/// Backend identity for logs/metrics/diagnostics.
pub enum BackendKind { Postgres, Iceberg, Graph, DataLake, Fake }

// ---- Base: universal to ANY store, whatever the data kind ----
// The `where StorageError: From<Self::Error>` lives HERE so every capability op and
// `Repository::ingest` gets the `?`-conversion into the currency for free.
// No `#[async_trait]`: the base has only the sync `backend()` — the async surface lives on the
// data traits (RawStore/GraphStore/FactStore), each of which carries its own `#[async_trait]`.
pub trait Storage: Send + Sync
where
    StorageError: From<Self::Error>,
{
    /// Impl-specific, rich error. Bounded above to convert into `StorageError`.
    type Error;

    /// Backend identity for logs/metrics/diagnostics. The base carries identity + the error
    /// currency ONLY (see below); it is not a lifecycle or data-op surface.
    fn backend(&self) -> BackendKind;

    // Deliberately NOTHING else here (STA-145 freeze). Lifecycle (health/migrate) is a
    // composition-root concern on the concrete type (§Startup); data ops and any
    // transaction/unit-of-work seam NEVER live on the base. A generic `Lifecycle: Storage`
    // trait is added later only if a generic bring-up consumer actually needs one.
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
    /// `FactSet` carries no identity (it is `entity`/`period`/`facts` — see `xbrl`), so the
    /// resolved `CompanyId` is passed alongside it; this is why identity also lives on
    /// `IngestionUnit`, not inside `FactSet`.
    ///
    /// **Upsert-by-full-grain, never "replace the period's value" (STA-145 freeze).** The key
    /// includes `source_ref`, so a 10-K/A restatement *coexists* with the original rather than
    /// overwriting it — provenance is preserved and "which value wins" is a read-time selection
    /// (`hybrid_data_model.md` §2.1 multi-adapter rule, §6). No impl may collapse the grain.
    async fn upsert(&self, company: &CompanyId, facts: &FactSet) -> Result<(), Self::Error>;
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

    /// The live write path. **Grain (STA-145 freeze): one `IngestionUnit` per filing** — the unit
    /// of ingestion is a single filing's raw + graph-delta + facts, persisted as one unit. Impl
    /// owns atomicity.
    async fn ingest(&self, unit: IngestionUnit) -> Result<(), StorageError>;
}
```

A default-ish `ingest` body composes the tiers, and every `?` converts via the base bound:

```rust
async fn ingest(&self, unit: IngestionUnit) -> Result<(), StorageError> {
    self.raw().append(&unit.raw).await?;                    // <Self::Raw as Storage>::Error   -> StorageError
    self.graph().upsert(&unit.graph).await?;                // <Self::Graph as Storage>::Error -> StorageError
    self.facts().upsert(&unit.company, &unit.facts).await?; // <Self::Facts as Storage>::Error -> StorageError
    Ok(())
}
```

> ⚠️ **This composed body realizes only the *weak* contract** — each tier-call acquires its own
> connection and commits independently, so a mid-unit failure leaves earlier tiers committed. It is
> the correct shape for a mixed-engine `Repository` (and for fakes), and **wrong for a co-located
> one**: `PostgresRepository` must NOT implement `ingest` this way — see
> [Transaction ownership](#tx-ownership).

> Note the return-type split: **capability ops return the rich `Self::Error`** (caller gets native
> detail, or converts to the currency at will); **`Repository::ingest` returns `StorageError`**
> because it unifies three different store errors and is where cross-cutting logic (retry, logging)
> reads classification.

## `StorageError` — the currency

A concrete enum that **classifies without discarding**: the `From<ImplError>` impl decides the
variant; the original backend error rides along as `source()`.

Naming and messages follow the project error conventions (AGENTS.md § "Error Naming Conventions" /
"Error Display Format"): Adjective-First / Failed-First variant names, each `#[error]` prefixed with
its `[VariantName]` and chained with `Caused by: {0}` (leaf variants use `Reason:` or nothing).

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Transient — safe to retry (connection drop, serialization failure, timeout).
    #[error("[UnavailableStorage] Storage backend is temporarily unavailable, Caused by: {0}")]
    UnavailableStorage(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Write conflicts with existing data (unique violation, already-exists).
    #[error("[ConflictingWrite] Write conflicts with existing data, Caused by: {0}")]
    ConflictingWrite(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Requested item not present (leaf — no inner error to chain).
    #[error("[MissingRecord] Requested record not found")]
    MissingRecord,

    /// Data integrity / invariant violated (check constraint, SFAC-6 identity).
    #[error("[FailedIntegrityCheck] Data integrity or invariant violated, Caused by: {0}")]
    FailedIntegrityCheck(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Uncategorized backend failure.
    #[error("[FailedBackendOperation] Backend operation failed, Caused by: {0}")]
    FailedBackendOperation(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl StorageError {
    /// The one decision the ETL loop needs: retry vs dead-letter.
    #[must_use]
    pub const fn is_retryable(&self) -> bool { matches!(self, Self::UnavailableStorage(_)) }
}
```

Each backend impls `From<ItsError> for StorageError`, mapping (e.g.) a Postgres serialization
failure → `UnavailableStorage`, a unique-violation → `ConflictingWrite`, a check-constraint →
`FailedIntegrityCheck`. Fakes take the trivial route: `type Error = StorageError` (the reflexive
`From<StorageError>` satisfies the bound with zero boilerplate).

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

### Transaction ownership — the facade holds the tx, never the tier stores {#tx-ownership}

The unit-of-work seam is `ingest`, but *where the transaction lives* differs by impl, and getting
this wrong silently downgrades the strong contract to the weak one:

- **Co-located impl (`PostgresRepository`): `ingest` does NOT delegate to its own capability-trait
  methods.** Calling `self.raw().append()` → `self.graph().upsert()` → `self.facts().upsert()`
  yields **three independent commits** — a mid-unit failure leaves a torn unit (raw + graph
  committed, facts absent), which is exactly what the strong contract exists to rule out. Instead,
  `ingest` is implemented **directly against the shared pool**: open one transaction, run the three
  tier writes as **impl-private helpers taking `&mut PgTransaction`**, commit; any error → native
  rollback, the unit never happened. The transaction type appears only inside `storage-postgres`,
  in private signatures — principle 6 (no `type Transaction` / `begin()` on any trait) is untouched.
  The public capability-trait methods on the Postgres stores remain for **standalone/replay use**
  (each wrapping its own short tx); they are simply not `ingest`'s building blocks.
- **Mixed-engine impl: there is no cross-engine rollback, by design.** No transaction spans
  Postgres + Neo4j + Iceberg, and the design does not simulate one (no 2PC, no compensating
  actions). The composed tier-by-tier body above is the *correct* implementation here: the **raw
  write is the commit point** — if it fails, `ingest` returns `Err` and the unit cleanly never
  happened; if a later projection write fails, nothing is undone — graph/fact writes are
  **idempotent by natural key**, so the unit is retried until convergent, and abandoned halves are
  caught by the §8 reconciliation job and replayed. Effectively the raw tier is a transactional
  outbox and the projections are its at-least-once, idempotent consumers.
- **Fakes:** `FakeRepository::ingest` records the `IngestionUnit` and returns `Ok` — it neither
  composes tier-calls nor models a transaction, keeping tests pinned to the weak contract.

Rule of thumb for review: **atomicity is the composing facade's private business.** If a tier
store's public method shows up inside `ingest`'s body in a co-located impl, or a transaction type
shows up in any trait signature, the boundary has been drawn wrong.

## Domain types the traits reference (co-design with §14.D.1 core types)

Type ownership follows the crate split: **domain vocabulary lives in `xbrl`** (a `FactSet` is a
`FactSet` regardless of persistence — it must not depend on a crate named `storage`), while the
**ingest-contract DTOs and query/report shapes live in `storage`** (they exist only to cross the
persistence seam):

```rust
// ---- `storage` crate: the ingest contract + query/report shapes ----

/// Everything derived from ONE filing, ready to persist as a unit. The pipeline builds this;
/// it never knows where or how it lands.
pub struct IngestionUnit {
    pub company: CompanyId, // resolved identity (§4) — `FactSet` carries none, so it lives here;
                            //   also the key the graph delta and raw filing attach under
    pub raw:   RawFiling,   // verbatim — SoT
    pub graph: GraphDelta,  // company/identifier/filing/concept nodes + edges to upsert
    pub facts: FactSet,     // canonical facts at the §6 grain (type from `xbrl`)
}

pub struct RawFiling { /* cik, accession, native tag, taxonomy_version, value, unit, period... */ }

/// An idempotent set of graph nodes + edges to upsert, mirroring `hybrid_data_model.md` §5.1/§5.2.
/// Structural edges are adapter-verifiable; **claim edges carry the uniform claim envelope**
/// (`source`, `as_of`, `observed_at`, `verifiability`) and are **append-only — never destructively
/// merged** (conflicting claims coexist; the winner is a read-time policy, §5.2). One filing's
/// delta is thus: the `Company` + its `Identifier`s (axiomatic ring), the `Filing`/`Concept`/`Period`
/// nodes and structural edges (`HAS_IDENTIFIER`/`HAS_FILING`/`COVERS_PERIOD`/`REPORTS_CONCEPT`), plus
/// any relationship *claims* the filing asserts.
pub struct GraphDelta {
    pub nodes:  Vec<GraphNode>,  // Company · Identifier · Filing · Concept · Period (idempotent by key)
    pub edges:  Vec<GraphEdge>,  // structural edges + claim edges (see below)
}
pub enum GraphNode {           // key per §5.1:
    Company(/*company_id*/), Identifier(/*scheme,value*/), Filing(/*regulator,native_id*/),
    Concept(/*canonical_element*/), Period(/*kind,key*/),
}
pub enum GraphEdge {
    Structural { /* kind: HAS_IDENTIFIER | HAS_FILING | COVERS_PERIOD | REPORTS_CONCEPT | …, from, to */ },
    Claim {      /* kind: LISTED_ON | IN_INDUSTRY | SUBSIDIARY_OF | OWNS_STAKE_IN, from, to, payload,
                    envelope: ClaimEnvelope */ },
}
/// The §5.2 claim envelope — provenance + point-in-time + trust, attached to every claim edge.
pub struct ClaimEnvelope { /* source, as_of, observed_at, verifiability: Verified|Reported|Alleged */ }

pub struct FactQuery { /* element(s), period range, dimension filter, ... */ }
pub struct CompletenessReport { /* missing periods / missing required concepts */ }
pub struct OwnershipTree { /* root + edges */ }

// ---- `xbrl` crate (domain vocabulary — referenced, not defined, by `storage`) ----

pub struct FactSet { /* Vec<CanonicalFact> for one company/filing */ }
pub struct CanonicalFact { /* §6 grain: company_id, canonical_element, value, unit, period... */ }
pub struct CompanyId(/* LEI preferred, CIK fallback */);
pub struct FiscalYear(u16);
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
}
// ... impl RawStore for FakeRawStore (real bodies for append/scan), etc.
// Fakes implement the read methods for real (they are the test oracle); Postgres reads stay todo!().

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
crate-internal, so they cannot cross a crate boundary. Because the traits live in the new `storage`
crate (below) but the consuming `Load` state lives in `sec`, the fakes must be reachable from both →
ship them **feature-gated in `storage`** (`#[cfg(feature = "fakes")]`), pulled into `sec`'s dev-deps.
This is the one place we deviate from the skill's "fakes in `tests/fixtures`" convention, and the
deviation is *forced by the crate split*, not preference.

## Crate topology (from the `storage`-crate decision)

The persistence ports must not depend on any backend, or "storage-agnostic" is a lie the compiler
won't catch — and the domain vocabulary must not depend on the ports (a `FactSet` is domain, not
persistence). So the decision implies a small topology, not one crate:

```
xbrl              domain vocabulary: CanonicalElement, FactSet, CanonicalFact,
  ▲  ▲             CompanyId/Lei, Invariant, Period, Unit.  No async, no I/O.
  │  │
  │  storage      persistence ports: Storage/RawStore/GraphStore/FactStore/Repository traits
  │   ▲  ▲         + StorageError + ingest DTOs (IngestionUnit, RawFiling, GraphDelta)
  │   │  │         + feature-gated fakes.  deps: xbrl, async-trait, futures, thiserror.  NO sqlx.
  │   │  │
  │   │  storage-postgres   PostgresRepository + the three Postgres stores. deps: storage + sqlx.
  │   │   ▲
  └───┴───┼────── sec        (SEC adapter → IngestionUnit; the Load state holds the concrete store
          │                   — via `storage` traits only, never `storage-postgres` directly)
          └────── the binary / composition root wires PostgresRepository and injects it
```

Only the composition root ever names Postgres; `sec` / `state_machine` / `xbrl` see nothing but
`storage` traits. This only holds if the Postgres impl is a *separate crate* — otherwise `sec`'s
dep graph pulls in sqlx transitively and the boundary rots.

> **Naming note (2026-08-08 revision).** This crate was called `domain` in earlier drafts. Renamed
> `storage`: it holds the persistence *ports*, not the domain vocabulary — that stays in `xbrl`,
> which `storage` depends on. The `storage` / `storage-postgres` pair also reads as one family
> (cf. `sqlx` / `sqlx-postgres`).

## DI wiring (same as SecClient — concrete, no `dyn`)

Inject the concrete `Repository` type into the `Load` state's **context**, exactly as `SecClient` is
a concrete field on `ExtractSuperStateContext`. Production wires `PostgresRepository`; tests wire
`FakeRepository`. No `dyn`, no trait objects — consistent with the non-object-safe `State` trait.
`Load` calls `store.ingest(unit)`. `CreateFinancialStatements` changes from emitting the placeholder
output to producing `FactSet`s that feed the `IngestionUnit` — the "retire `sec::CompanyData`" work.

> **See also** `load_superstate_design.md` — the Load `SuperState` designed against these ports
> (sub-states, the `FinancialStatementRepository` / `LeiResolver` ports, adapters, and UML).

### Load-port reconciliation (STA-145 — pinned)

`load_superstate_design.md` names two Load-facing ports; STA-145 pins how they line up with the
frozen storage traits so the two docs cannot drift:

- **`FinancialStatementRepository` is the Load-*facing* port, not a second storage trait.** It speaks
  the **application** domain (`FinancialStatements`), keeping the Load state thin. Its concrete
  adapter (`PostgresFinancialStatementRepository`) is *where the storage `Repository` facade + tier
  stores live*: a pure `FinancialStatements → IngestionUnit` mapper feeds `Repository::ingest`. So
  the storage `Repository` is the **implementation** of `FinancialStatementRepository::store`, never
  something the Load state sees. (This is the recommended placement — mapping in the adapter,
  `load_superstate_design.md` §6.A.)
- **`LeiResolver` stays a distinct Load port.** Identity resolution (CIK → `CompanyId`) is a separate
  outbound concern with its own adapters (static map now, GLEIF later); it is **not** a storage
  trait and does not fold into `Repository`. It runs in `ResolveCompanyIdentity` and produces the
  `CompanyId` that `IngestionUnit` then carries.

Net: one storage contract (this doc), two Load ports (that doc); the storage `Repository` is an
adapter internal of one of them.

## What must NEVER leak through a trait (review checklist)

| Leak | Why it kills the swap | Instead |
|---|---|---|
| `sqlx::Transaction` / conn type in a signature | binds callers to Postgres | atomicity inside `ingest` |
| SQL/Cypher strings in or out | binds to a query engine | typed `FactQuery` / domain structs |
| a store's `Self::Error` in `Repository::ingest`'s signature | re-fragments the currency | `ingest -> StorageError`; convert at the seam |
| `sqlx::Error` naked (unconverted) escaping a store | forces callers to look like Postgres | rich `type Error` that `From`-converts into `StorageError` |
| store-specific pagination cursors | leaks engine internals | domain-level page token |
| `type Transaction` / `begin()` on `Storage` | every fake + future graph DB must model it | keep base = identity (`backend()`) + `Error` currency only |
| lifecycle (`health`/`migrate`) on a data trait | every fake + future engine must model it | call inherent methods on the concrete impl at the root (§Startup) |

## Payoff of the base trait — generic over `impl Storage`

The base trait earns its place on **two** things, both frozen: the `type Error` +
`StorageError: From<Self::Error>` bound (so every capability op and `ingest` gets `?`-conversion into
the one currency for free) and `backend()` identity. No `dyn`, so anything written against
`impl Storage` is a generic function called per concrete store, blind to data kind:

```rust
// Metrics/tracing decorator, readiness surface, "which backends are we on" diagnostic — each
// written once against `impl Storage`. A future Iceberg `FactStore` is picked up for free.
fn describe<S: Storage>(s: &S) -> BackendKind { s.backend() }
```

### Startup / lifecycle lives at the composition root, not on the trait {#lifecycle}

Bring-up (schema migrate + health check) is **not** a trait method (STA-145 freeze — it is off the
base). The composition root knows the concrete `PostgresRepository`, so it calls that type's own
inherent `migrate()` / `health()` at startup:

```rust
// In `main` / the composition root — concrete type, no trait needed:
let repo = PostgresRepository::connect(&cfg).await?;
repo.migrate().await?;      // inherent method on the concrete impl
repo.health().await?;
```

Rationale: lifecycle is a root concern that never appears in pipeline/state code, and putting it on
the base would force every fake and every future engine (a graph DB, Iceberg) to model it for no
caller benefit. If a *generic* fleet bring-up consumer ever materializes, introduce a
`Lifecycle: Storage` trait then — the base stays minimal until then.

## Revised implementation order (collapses handoff steps 1 & 3)

1. **`storage` crate: ports + `StorageError` + ingest DTOs + feature-gated fakes** — the DI
   contract. (DTOs and traits are one ticket: the DTOs are the traits' vocabulary; the domain
   vocabulary itself stays in `xbrl`, which `storage` depends on.)
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

## Resolved by STA-145 (the frozen decisions)

The open questions from earlier drafts are now closed. Each resolution is baked into the frozen
signatures above; recorded here so the reasoning survives.

- **Method inventory — CLOSED.** `ingest` is the **only** live-write entry point the ETL calls.
  The per-tier `append` / `upsert` (+ `RawStore::scan`) exist for standalone replay/rebuild, not for
  the pipeline to sequence. The full frozen inventory and per-method implementation status are the
  table under [Traits](#traits-frozen--sta-145).
- **`health()` / `migrate()` on the base — CLOSED: no.** Base `Storage` = `type Error` + `backend()`
  only. Lifecycle is a composition-root concern on the concrete type (§Startup). A `Lifecycle: Storage`
  trait is a *later* addition, gated on a real generic-bring-up consumer.
- **Unit-of-ingestion grain — CLOSED: one `IngestionUnit` per filing.** Frozen on
  `Repository::ingest`.
- **Reads now or later — CLOSED: declare now, implement lazily.** Read signatures
  (`completeness` / `ownership_tree` / `query`) are frozen *as vocabulary* so the trait shape is
  stable; only the fakes get real bodies in STA-139, Postgres reads stay `todo!()` until a
  screener/API consumer is cut. `RawStore::scan` is the exception — it is implemented now because
  replay/rebuild needs it.
- **`FactStore` write/read split — CLOSED: one trait now.** The future Postgres-landing +
  Iceberg-scan split (§12b.3) stays hidden behind it; callers never learn there are two engines.
- **Restatement invariant — CLOSED: upsert-by-full-grain.** `FactStore::upsert` keys on the full
  grain incl. `source_ref`; a 10-K/A coexists with the original, "which value wins" is a read-time
  selection. Frozen as a doc-comment on the method so no impl collapses the grain.
- **Identity on the ingest DTO — CLOSED.** Because `xbrl::FactSet` carries no `CompanyId`, the
  resolved identity lives on `IngestionUnit.company` and is passed to `FactStore::upsert(company, …)`.

### Still deferred (with the trigger that re-opens each)

These are *not* part of the frozen v1 — they wait for a concrete consumer, by design:

- **Read-side query surface, fully shaped** → when a **screener/API** consumer exists. Its return
  DTOs (`CompletenessReport`, `OwnershipTree`, richer `FactQuery`) are named now but will be
  finalized *by* that consumer; freezing them earlier would guess at it.
- **Bulk vs incremental write facade (`BulkLoad`)** → when a **backfill/migration** consumer exists
  (see "Deferred — update semantics" above). Orthogonal to the raw/graph/facts split.
- **`Lifecycle: Storage` trait** → when a **generic** fleet-bring-up consumer needs migrate/health
  behind `impl Storage` rather than on the concrete root.

## Revision note — what changed from the first draft

- **`Repository` (composition via associated types)** replaces `FinancialDataStore: RawStore +
  GraphStore + FactStore` (inheritance). Reason: has-a makes the engine *mixture* the natural case;
  is-a forced co-location. Matches the `SecClient` / `InnerClient` house pattern.
- **Per-store `type Error`** (rich, impl-specific) bounded `StorageError: From<Self::Error>` on the
  base `Storage`, replacing the single naked `StorageError` everywhere. Keeps the house `type Error`
  pattern *and* uniform retryability; `?` works because the bound is `From`-shaped.
- **`StorageError` carries `source()`** (boxed) and classifies — no backend detail discarded.
- **No `dyn` anywhere** — was `Arc<dyn FinancialDataStore>` injection + `&[&dyn Storage]` bring-up;
  now concrete/generic, consistent with the non-object-safe `State` trait. Anything written against
  `impl Storage` (metrics/diagnostics) is a generic fn called per tier. (Startup migrate/health is a
  concrete-type call at the root — see the STA-145 entry below.)
- **Crate topology + feature-gated fakes** spelled out as the consequence of the ports-crate
  decision.
- **(2026-08-08)** Crate renamed **`domain` → `storage`**; domain vocabulary (`FactSet`,
  `CanonicalFact`, `CompanyId`, …) stays in `xbrl`, which `storage` depends on — see the naming
  note under Crate topology.
- **(2026-08-08)** New **Transaction ownership** subsection (§tx-ownership): the composing facade
  owns atomicity; a co-located impl must not build `ingest` from its own capability-trait methods.
- **(2026-08-08)** Header decision updated: **no physical deployment chosen** — abstraction-first;
  the physical choice is deferred behind these ports until a measured trigger forces it.
- **(2026-08-10, STA-145 — signatures FROZEN)** The trait inventory is closed and the doc is the
  contract STA-139 scaffolds verbatim. Specifically:
  - Base `Storage` **loses `health()` / `migrate()`** — now `type Error` + `backend()` only;
    lifecycle moved to the composition root on the concrete impl (§Startup). Review checklist +
    fakes + the base-trait payoff section updated to match.
  - `FactStore::upsert` **takes `company: &CompanyId`** (since `xbrl::FactSet` carries no identity)
    and gained the **restatement / upsert-by-full-grain** doc-comment.
  - `IngestionUnit` **carries `company: CompanyId`**; `ingest`'s composed body updated.
  - `GraphDelta` **fleshed out to the §5 three-ring model** (Company + Identifier nodes, structural
    vs. **claim edges with the claim envelope**, append-only).
  - Grain frozen: **one `IngestionUnit` per filing**.
  - "Open questions" replaced by **"Resolved by STA-145"** + an explicit **deferred-with-trigger**
    list (read surface, `BulkLoad`, `Lifecycle` trait).
  - **Load-port reconciliation** pinned: `FinancialStatementRepository` = Load-facing port whose
    adapter *contains* the storage `Repository`; `LeiResolver` stays a distinct Load port.
- **(2026-08-10)** `StorageError` **realigned to the project error conventions** (AGENTS.md): variant
  names are now Adjective-First / Failed-First (`Unavailable`→`UnavailableStorage`,
  `Conflict`→`ConflictingWrite`, `NotFound`→`MissingRecord`, `Integrity`→`FailedIntegrityCheck`,
  `Backend`→`FailedBackendOperation`), and each `#[error(...)]` carries its `[VariantName]` prefix +
  `Caused by: {0}` chaining (matching `xbrl`/`sec` error types). The earlier bare-name, lowercase
  messages would have violated conventions once STA-139 scaffolded them verbatim.
- **(2026-08-10) — Consolidated Design pass (supersedes the frozen v1 above).** Added the
  [Consolidated Design](#consolidated-design-current--2026-08-10) section as the authoritative shape.
  Substantive changes from the STA-145 freeze:
  - **`Repository` is now neutral** — `type Record` + `persist` only; backend-blind. The
    `type Raw/Graph/Facts` triad + accessors moved out of the port.
  - **`SecRepository<R,G,F>` (in `sec`)** composes the triad and owns SEC-specific raw schema/mapping;
    `Repository` and `Backend` meet only inside `persist`. Future `EsefRepository` shares the
    canonical graph/facts tiers.
  - **`ingest` → `persist`**, **`IngestionUnit` → `FilingRecord`** (in `sec`), **base `Storage` →
    `Backend`** with **`backend()` → `kind()`**.
  - **`StorageError` → `error::ErrorKind`** — a `Clone`/`Eq`/`Ord`/`Hash` **value type** carrying
    `Reason` strings (no boxed source), matching the `sec` error hierarchy; up/down-cast (`From` /
    `TryFrom` / `DowncastNotPossible`) happens at the `sec` seam where it becomes a `State` leaf.
  - **`BackendKind`** gained `Memory` / `FileSystem` (in-memory & filesystem backends are first-class).
  - **Fakes** are per-crate `#[cfg(test)]` in each crate's `tests/fixtures/` (house convention),
    reversing the earlier "feature-gated fakes in `storage`" note; promote to a shared testkit only
    on a second consumer.
  - Deferred to STA-139: read surface + CQRS split, `RawStore` payload genericity, `completeness`
    placement, retry/DLQ (decorator + reconciliation + poison-only DLQ, driver-level).
- **(2026-08-10) — error model reworked to mirror the `sec` error hierarchy.** Replaced the flat
  `ErrorKind` with an **operation-classed** hierarchy built exactly like `sec/src/lib/error/`:
  `ErrorKind` (top union) = `Read(ReadError)` / `Write(WriteError)`; a shared `BackendError`
  (`Unavailable`/`Failed`, `is_retryable()`) embedded in each; module-per-level, markers inline vs
  rich leaves in their own module, `From` upcast / `TryFrom` downcast, `DowncastNotPossible` sentinel
  on the top, full `implements_*` test suite per level. **Methods return the narrow class** —
  `persist -> WriteError`, future `query -> ReadError` — so illegal states are unrepresentable; the
  union `ErrorKind` is what the retry decorator + `sec` seam consume. This **drops `Backend::type
  Error`** (stores return the classes directly; the value-type/`reason`-string decision already
  flattened rich detail, making the associated error moot). Built write-side first; `ReadError`
  arrives with the STA-139 read methods.
