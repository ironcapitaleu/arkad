# Storage Abstraction — Trait Design

> **No physical deployment option is chosen.** The `storage` crate's ports (this doc) abstract
> physical storage completely. The physical choice (Option A, B, or any mixture —
> `hybrid_data_model.md` §14) waits until a measured trigger forces it.
>
> The [Design](#design) section holds the shape of the ports. The sections after it give the
> rationale. They use the names `ingest`, `Storage`, `StorageError`, and a composing `Repository`.
> Read `persist`, `Backend`, `ErrorKind`, and `SecRepository`.

## Design

### Naming & structure

| Element | Decision |
| --- | --- |
| **crate** | `storage` — owns the word, so no trait is named `Storage` |
| **facade** | `Repository` — *neutral*: `type Record` + `persist`, backend-blind |
| **write method** | `persist(record)` |
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
    │   ├── backend_error/mod.rs # BackendError, BackendCause (is_retryable planned)
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
  backend_error/mod.rs  → BackendError { UnreachableStorage{cause}, FailedOperation{cause}, UnauthorizedAccess{cause} }
                          // cause: BackendCause { Reason(String), FailedFilesystemOperation(…) }
  // a RICH leaf gets its own module (e.g. missing_read_permission/), exactly as sec splits `invalid_cik_format`
```

**Per-level conventions — identical to sec:**

- `#[non_exhaustive]` + `#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]`. **Value
  type** — no `Box<dyn Error>`. A backend error holds a `BackendCause`: a `reason` string, or a
  wrapped error that is itself a value type (`Clone`, `Eq`, `Hash`, `Ord`). This is exactly what lets
  each level be *contained in* the level above it. An error that is not a value type, such as
  `sqlx::Error`, is flattened to a `reason` string at the conversion boundary.
- **Pass-through layers print no segment.** `ErrorKind::Read`, `ErrorKind::Write`,
  `ReadError::Backend`, and `WriteError::Backend` add no fact, so they use `#[error(transparent)]`.
  A `BackendError` with a wrapped `FailedFilesystemOperation` passes it through, because the
  filesystem error already names the kind. A `BackendError` with a reason stays a leaf. See
  `AGENTS.md` ("Error Display Format").
- **Markers stay inline** on the enum with `#[error("[…] …, Reason: '…'")]`. `BackendError` is the
  exception: its `Display` and `Error` are written by hand, because its `source()` depends on its
  `BackendCause`. **Rich leaves get their
  own module** and are wrapped (`Read(ReadError)`-style). Same split as sec's `State::InvalidInput`
  (inline) vs `State::InvalidCikFormat(_)` (module). Today's `WriteError` and `ReadError` causes are all markers → inline.
- **Upcast** = `From<Inner> for Outer` — infallible, `?`-able: `MissingReadPermission → ReadError →
  ErrorKind`. So a read method returns `ReadError` and anything `Into<ReadError>` flows in via `?`.
- **Downcast** = `TryFrom<Outer> for Inner`, `type Error = ErrorKind`, returns
  `ErrorKind::DowncastNotPossible` on mismatch (skip-level allowed, e.g. `TryFrom<ErrorKind> for
  BackendError`). This is the *fallible* direction — not `Into`.
- **Naming — short variants, suffixed types (the one deliberate deviation from sec).** Variants stay
  short (`ErrorKind::Write`, `WriteError::Backend`) since they're namespaced and can't clash; the
  **types carry the `Error` suffix** (`ReadError` / `WriteError` / `BackendError`). Unlike sec's
  suffix-less `State` / `Transition`, these three words collide: bare `Read` / `Write` shadow
  `std::io::{Read, Write}` and `std::fmt::Write`, and bare `Backend` shadows the `Backend` store
  trait — and only *types* escape into `use`, so only they need disambiguating. Do **not** simplify
  the types back to bare `Read` / `Write` / `Backend`.

**Method return types — the narrow class, not the union:**

```rust
async fn persist(&self, r: Self::Record) -> Result<(), WriteError>;   // never MissingRecord
async fn query(&self, …)                 -> Result<_,  ReadError>;    // never ConflictingWrite   (STA-139)
```

The store methods return the classes **directly** (`append -> Result<(), WriteError>`); the Postgres
impl maps `sqlx::Error → WriteError::Backend(BackendError::failed_operation(reason))` at its own boundary.
A `BackendCause` holds either that reason or a wrapped error. A `BackendError` with a wrapped error
prints it as it is, and `source()` forwards to the source of the wrapped error. To reach the wrapped
error, match on `BackendError::cause()`. The filesystem backend wraps its `FailedFilesystemOperation`.
This **drops the earlier `Backend::type Error` associated error** — the value-type decision keeps
backend errors as values, so the associated type was buying flexibility we don't use.
`ErrorKind` (the union) is what the **shared consumers** take — the retry decorator (`is_retryable()`, planned)
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
- **`sec` fixtures for binaries:** a binary's tests link the library built without `cfg(test)`. So
  `sec` also compiles its fixtures under the `test-fixtures` feature, and names itself as a
  dev-dependency with that feature (STA-378). Only test targets turn the feature on.

### Media type of a stored document

`DocumentMetadata.media_type` holds a `MediaType`, a checked newtype in `storage`. Media types are
an open set, so `MediaType` is not an enum. It checks the form `type/subtype`, not a fixed list.

- **Normalization.** `MediaType::new` drops the parameters after `;`, trims whitespace, and
  lowercases the text. Then the type and the subtype must each be a non-empty RFC 9110 token
  other than the wildcard `*`.
  `Display` and serde write the normalized text.
- **Read.** Deserializing runs `MediaType::new`. A stored `media_type` that is not a media type
  fails to read. `persist` rewrites such a metadata file, because its JSON differs from the new
  metadata.
- **No mapping trait between `ContentType` and `MediaType`.** `storage` does not depend on `sec`,
  and `sec` keeps `ContentType` to classify responses. A private function in `raw_document`
  builds the `MediaType` from the `Display` text of the `ContentType`.
- **Fallback.** If that text is not a media type, as for `ContentType::Unknown` or a malformed
  `ContentType::Other`, the media type is `application/octet-stream` (RFC 9110, section 8.3).
  `from_response` and `from_file` stay infallible. Today every body kind of an `SecResponse`
  rejects both variants, so a stored raw document never takes the fallback.

### Deferred to STA-139 (marked, not decided)

1. **Read surface** (`scan` / `completeness` / `ownership_tree` / `query`) + the CQRS read/write split.
   The listing of a repository reopens this item for the listing only, and replaces `scan`. See
   [`repository_listing.md`](../../../storage/design/data_model/repository_listing.md).
2. **`RawStore` genericity** (`type Item`?) — raw is the one regulator-specific tier.
3. **`completeness` placement** — straddles graph structure and fact presence.
4. **Retry / DLQ** — a `RetryingRepository` decorator (reads `is_retryable()`); reconciliation for
   post-raw projection gaps; a DLQ only for poison (non-retryable) records; all at the driver level,
   never in the state machine. For the MVP, the silver load needs no reconciliation job that
   replays projections. See the note at [Transaction ownership](#tx-ownership).

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

> **Note:** [Amendment 1 of ADR 0001](../../../docs/adr/0001-mvp-database.md#amendment-1-load-reads-silver) proposes that `persist` writes the raw store only.
> A load step per silver unit then writes the graph and fact tiers in one Postgres transaction. The
> co-located rule below applies to that load step. The mixed-engine path and its §8 reconciliation
> job have no use in the MVP. A failed load leaves no ledger row, so the next run loads the unit
> again.

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

Only the composition root ever names Postgres; `sec` / `state_maschine` / `xbrl` see nothing but
`storage` traits. This only holds if the Postgres impl is a *separate crate* — otherwise `sec`'s
dep graph pulls in sqlx transitively and the boundary rots.

## DI wiring (same as SecClient — concrete, no `dyn`)

Inject the concrete `Repository` type into the `Load` state's **context**, exactly as `SecClient` is
a concrete field on `ExtractCompanyFactsSuperStateContext`. Production wires `PostgresRepository`;
tests wire `FakeRepository`. No `dyn`, no trait objects — consistent with the non-object-safe
`State` trait.
`Load` calls `store.ingest(unit)`. `CreateFinancialStatements` changes from emitting the placeholder
output to producing `FactSet`s that feed the `IngestionUnit` — the "retire `sec::CompanyData`" work.

> **See also** `load_superstate_design.md` — the Load `SuperState` designed against these ports
> (sub-states, the `FinancialStatementRepository` / `LeiResolver` ports, adapters, and UML).

### Load-port reconciliation

`load_superstate_design.md` names two Load-facing ports. They line up with the storage traits as
follows:

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

The base trait earns its place on **two** things: the `type Error` +
`StorageError: From<Self::Error>` bound (so every capability op and `ingest` gets `?`-conversion into
the one currency for free) and `backend()` identity. No `dyn`, so anything written against
`impl Storage` is a generic function called per concrete store, blind to data kind:

```rust
// Metrics/tracing decorator, readiness surface, "which backends are we on" diagnostic — each
// written once against `impl Storage`. A future Iceberg `FactStore` is picked up for free.
fn describe<S: Storage>(s: &S) -> BackendKind { s.backend() }
```

### Startup / lifecycle lives at the composition root, not on the trait {#lifecycle}

Bring-up (schema migrate + health check) is **not** a trait method — it is off the
base. The composition root knows the concrete `PostgresRepository`, so it calls that type's own
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

## Implementation order

1. **`storage` crate: ports + `StorageError` + ingest DTOs + feature-gated fakes** — the DI
   contract. (DTOs and traits are one ticket: the DTOs are the traits' vocabulary; the domain
   vocabulary itself stays in `xbrl`, which `storage` depends on.)
2. **`storage-postgres` crate: `PostgresRepository`** implementing the traits (`ingest` = one tx).
3. **SEC adapter** — raw parse + resolution map + CIK→LEI, producing `IngestionUnit`s.
4. **Wire `CreateFinancialStatements` → `FactSet` → `Load` calls `ingest`**; retire `sec::CompanyData`.

## Deferred — update semantics: bulk vs incremental (orthogonal axis)

This is a *second axis*,
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

## Decisions

- **Method inventory — CLOSED.** `ingest` is the **only** live-write entry point the ETL calls.
  The per-tier `append` / `upsert` (+ `RawStore::scan`) exist for standalone replay/rebuild, not for
  the pipeline to sequence.
- **`health()` / `migrate()` on the base — CLOSED: no.** Base `Storage` = `type Error` + `backend()`
  only. Lifecycle is a composition-root concern on the concrete type (§Startup). A `Lifecycle: Storage`
  trait is a *later* addition, gated on a real generic-bring-up consumer.
- **Unit-of-ingestion grain — CLOSED: one `IngestionUnit` per filing.** Fixed on
  `Repository::ingest`.
- **Reads now or later — CLOSED: declare now, implement lazily.** Read signatures
  (`completeness` / `ownership_tree` / `query`) are fixed *as vocabulary* so the trait shape is
  stable; only the fakes get real bodies, Postgres reads stay `todo!()` until a
  screener/API consumer is cut. `RawStore::scan` is the exception — it is implemented now because
  replay/rebuild needs it.
- **`FactStore` write/read split — CLOSED: one trait now.** The future Postgres-landing +
  Iceberg-scan split (§12b.3) stays hidden behind it; callers never learn there are two engines.
- **Restatement invariant — CLOSED: upsert-by-full-grain.** `FactStore::upsert` keys on the full
  grain incl. `source_ref`; a 10-K/A coexists with the original, "which value wins" is a read-time
  selection. Fixed as a doc-comment on the method so no impl collapses the grain.
- **Identity on the ingest DTO — CLOSED.** Because `xbrl::FactSet` carries no `CompanyId`, the
  resolved identity lives on `IngestionUnit.company` and is passed to `FactStore::upsert(company, …)`.

### Still deferred (with the trigger that re-opens each)

These wait for a concrete consumer, by design:

- **Read-side query surface, fully shaped** → when a **screener/API** consumer exists. Its return
  DTOs (`CompletenessReport`, `OwnershipTree`, richer `FactQuery`) are named now but will be
  finalized *by* that consumer; freezing them earlier would guess at it.
- **Bulk vs incremental write facade (`BulkLoad`)** → when a **backfill/migration** consumer exists
  (see "Deferred — update semantics" above). Orthogonal to the raw/graph/facts split.
- **`Lifecycle: Storage` trait** → when a **generic** fleet-bring-up consumer needs migrate/health
  behind `impl Storage` rather than on the concrete root.
