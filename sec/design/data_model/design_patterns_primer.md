# Design Patterns Primer — Separating Business Logic from Storage

> **Background / shared-vocabulary doc for STA-130.** This is *not* normative — the binding
> design lives in `storage_traits_design.md` and `hybrid_data_model.md` §14.F. The purpose here is
> a common vocabulary so we can argue about the design precisely ("that's a Unit-of-Work concern,
> not a Repository one") and onboard newcomers to *why* the storage layer is shaped the way it is.
>
> A runnable Python illustration of the arkad-specific combination is in
> **`design_patterns_demo.py`** (`python3 design_patterns_demo.py`). The examples below are Python
> for readability; each pattern also notes its **arkad (Rust)** mapping.

A good architecture separates three things:

1. **What the application does** — business logic (our ETL states, resolution, validation).
2. **How data is stored** — Postgres / Iceberg / a graph DB / a data lake.
3. **How operations are performed** — algorithms, external services (SEC HTTP, GLEIF).

The patterns below are the tools for keeping those apart. Most of them compose; the last section
shows the specific combination arkad's storage layer uses.

---

## Dependency Injection (DI)

**What:** Construct objects from the outside and depend on abstractions, not concretions. Instead of
`OrderService(PostgresRepository())`, write `OrderService(repository)` and let the caller decide.

**arkad:** Core. The concrete store is injected into a state's context, exactly as `SecClient` is a
concrete field on `ExtractSuperStateContext` today. Production wires `PostgresRepository`; tests wire
a fake. The pipeline names no backend.

## Ports & Adapters (Hexagonal Architecture)

**What:** The core application defines interfaces (**ports**) it needs — `UserRepository`,
`PaymentGateway`, `EmailSender`. Infrastructure provides **adapters** that implement them —
`PostgresRepository`, `StripeGateway`, `SMTPMailer`. Dependencies always point *inward*: the core
never imports the adapters.

**arkad:** The **primary** pattern of the storage layer. `Storage` / `RawStore` / `GraphStore` /
`FactStore` / `Repository` are ports; `PostgresRepository`, a future `Neo4jGraphStore`, and the
in-memory fakes are adapters. Same shape the codebase already uses for `SecClient` (port) over
`InnerClient`.

## Clean Architecture (the dependency rule)

**What:** Concentric layers — UI → Application → Domain → Infrastructure — where source-code
dependencies only ever point *inward*. The domain knows nothing about the database.

**arkad:** Enforced by the **crate boundary**, not just convention: the `storage` crate (ports +
ingest DTOs + `StorageError`; domain vocabulary stays in `xbrl`) has **no `sqlx`**; the
`storage-postgres` crate depends on `storage`, never the reverse. Only the composition root (the
binary) names a concrete DB. The compiler enforces the dependency rule.

## Repository

**What:** Abstract persistence behind a collection-like interface for a domain type — `get(id)`,
`add(entity)`, `find(spec)` — so business logic never sees SQL.

```python
class UserRepository(ABC):
    @abstractmethod
    def get(self, id: str) -> User: ...
# SQLUserRepository / MongoUserRepository / InMemoryUserRepository
```

**arkad:** Used **on the read side**, precisely: `FactStore.query`, `GraphStore.completeness`,
`GraphStore.ownership_tree` are genuine repository-style queries by criteria. The *composing*
`Repository` trait borrows the name as familiar shorthand, but it is really a **persistence facade**
(see the synthesis below), not a single-aggregate collection. We keep the name; we don't overclaim
the strict DDD meaning.

## Data Mapper

**What:** Keep persistence *out* of domain objects (the opposite of Active Record's `user.save()`).
A separate mapper moves data between domain objects and rows.

**arkad:** The domain types (`RawFiling`, `FactSet`, `GraphDelta`, `CanonicalFact`) carry no SQL and
no persistence methods; each **adapter** does the mapping between them and its engine's rows/edges.
Domain stays persistence-ignorant.

## Unit of Work

**What:** Coordinate writes across multiple stores in a single transaction — everything commits or
everything rolls back.

```python
with unit_of_work as uow:
    uow.users.add(user)
    uow.orders.add(order)
    uow.commit()   # atomic
```

**arkad:** `Repository.ingest(unit)` is a **Unit-of-Work-flavored** write: raw + graph + facts land
as one unit. Crucially, atomicity lives **inside the adapter** (a co-located Postgres impl wraps all
three tier-writes in one `tx`) — we deliberately do **not** expose a `begin()`/`commit()` seam on the
ports, which would leak transactions into every fake and every future engine.

## CQRS (Command Query Responsibility Segregation)

**What:** Separate the write model from the read model when they differ significantly. Writes update
the source of truth; reads can come from views/Redis/Elasticsearch shaped for querying.

**arkad:** A strong fit (this is `hybrid_data_model.md` §12b: population-vs-query asymmetry). The
**write path** is `ingest` (one bundle in). The **read paths** are separate analytical/traversal
queries. The **raw store is the source of truth**; graph + facts are **read-optimized projections**
that can be rebuilt by replaying raw. That is why read-after-write on graph/facts is *not* promised.

## Service Layer

**What:** A layer holding application/business logic (validation, authorization, transactions,
coordinating repositories) so controllers/entry-points stay thin.

**arkad:** The ETL **states** are the service layer. The `Load` state orchestrates `ingest`;
`CreateFinancialStatements` produces the `FactSet` that feeds the `IngestionUnit`. They depend only
on the storage ports.

## Specification

**What:** Represent a business rule / query predicate as a reusable, composable object
(`Adult AND HighIncome AND GoodCredit`) instead of scattered `if` conditions.

**arkad:** Not used yet, but a natural evolution of the read side: `FactQuery` / `RawScan` are query
objects today; if filter composition grows, a specification-style API is the path (noted as an open
question in `storage_traits_design.md`).

## Strategy

**What:** Encapsulate interchangeable algorithms behind one interface; the caller picks one.

**arkad:** Adjacent, not core to storage. The clearest candidates are the deferred **bulk vs
incremental** write facades (two strategies for populating a tier) and concept resolution/mapping in
the SEC adapter. Kept out of the storage ports for now.

## Command

**What:** Represent an operation as an object (`DeleteUserCommand` + a handler), enabling queues,
retries, undo, auditing.

**arkad:** Not used in the storage layer directly, but `IngestionUnit` is command-like (a
self-contained "persist this" message), and the state machine itself is command-flavored. Worth
knowing when we design retry/backfill queuing later.

---

## What arkad's storage layer actually is (the synthesis)

Strip the labels and the storage layer is:

> **Ports & Adapters (hexagonal) at the core**, with a **CQRS split** — a Unit-of-Work-flavored
> `ingest` write port and repository-style read queries — over a **source-of-truth + projections**
> storage model, wired by **Dependency Injection** and kept honest by the **Clean-Architecture crate
> boundary**.

Mapping the pieces:

| Concern | Pattern | In arkad |
| --- | --- | --- |
| Decouple from the DB backend | **Ports & Adapters** | `Storage`/`RawStore`/`GraphStore`/`FactStore`/`Repository` traits ↔ concrete adapters |
| Enforce the boundary | **Clean Architecture** | `storage` crate (no `sqlx`) ← `storage-postgres` crate |
| Wire it | **Dependency Injection** | concrete store injected into the `Load` context |
| Atomic multi-tier write | **Unit of Work** | `Repository::ingest` (atomicity inside the adapter) |
| Reads ≠ writes | **CQRS** | `ingest` vs `query`/`completeness`/`ownership_tree`; raw = SoT, graph/facts = projections |
| Domain stays SQL-free | **Data Mapper** | `RawFiling`/`FactSet`/… have no persistence logic |
| Query the read side | **Repository** (read-side, proper) | `FactStore::query`, `GraphStore::completeness/ownership_tree` |

**Why not "just the Repository pattern"?** Repository is an OLTP, entity-centric pattern: a
collection of one aggregate root, read-modify-write, symmetric reads/writes. Our workload is
ETL + analytics + graph: three heterogeneous tiers, write-derived bundles (not mutated aggregates),
and deeply asymmetric reads/writes. Repository fits the *read side* and lends a familiar name; the
*whole* is better described as ports-and-adapters + CQRS over SoT/projections. We reach for the
pattern that fits the use case, and name the parts honestly.

See `design_patterns_demo.py` for a runnable, ~200-line Python model of exactly this combination.
