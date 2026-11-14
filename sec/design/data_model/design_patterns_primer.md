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

**arkad:** Core. A state's context holds the repository as a generic field, as it holds
`SecClient`. `StoreCompanyFacts` takes any `RawRepository`. A binary wires `FilesystemRepository`,
and a test wires `FakeWriteRepository`. The pipeline names no backend.

## Ports & Adapters (Hexagonal Architecture)

**What:** The core application defines interfaces (**ports**) it needs — `UserRepository`,
`PaymentGateway`, `EmailSender`. Infrastructure provides **adapters** that implement them —
`PostgresRepository`, `StripeGateway`, `SMTPMailer`. Dependencies always point *inward*: the core
never imports the adapters.

**arkad:** The **primary** pattern of the storage layer. `WriteRepository`, `ReadRepository`, and
`ListRepository` are ports. `FilesystemRepository` and the fakes are adapters. The planned Postgres
backend is one more adapter. Same shape the codebase uses for `SecClient` (port) over
`InnerClient`.

## Clean Architecture (the dependency rule)

**What:** Concentric layers — UI → Application → Domain → Infrastructure — where source-code
dependencies only ever point *inward*. The domain knows nothing about the database.

**arkad:** Enforced by the **crate boundary**, not just convention. The `storage` crate holds the
ports and the errors, and depends on no database driver. The planned `storage-postgres` crate
depends on `storage`, never the reverse. Only the composition root (the binary) names a concrete
backend ([ADR 0005](../../../docs/adr/0005-storage-ports-crate.md)).

## Repository

**What:** Abstract persistence behind a collection-like interface for a domain type — `get(id)`,
`add(entity)`, `find(spec)` — so business logic never sees SQL.

```python
class UserRepository(ABC):
    @abstractmethod
    def get(self, id: str) -> User: ...
# SQLUserRepository / MongoUserRepository / InMemoryUserRepository
```

**arkad:** The ports carry the name. `ReadRepository::get` reads one record by key, and
`ListRepository::list` lists the documents of a store. Queries by criteria wait for a consumer
(storage overview, "Planned"). Each repository holds one document type, not one aggregate root, so
the name does not claim the strict DDD meaning.

## Data Mapper

**What:** Keep persistence *out* of domain objects (the opposite of Active Record's `user.save()`).
A separate mapper moves data between domain objects and rows.

**arkad:** The domain types (`RawDocument`, `FactSet`) carry no SQL and no persistence methods.
Each **adapter** maps them to its medium, such as a file and its metadata file. Domain stays
persistence-ignorant.

## Unit of Work

**What:** Coordinate writes across multiple stores in a single transaction — everything commits or
everything rolls back.

```python
with unit_of_work as uow:
    uow.users.add(user)
    uow.orders.add(order)
    uow.commit()   # atomic
```

**arkad:** The planned load step is **Unit-of-Work-flavored**: the graph rows, the fact rows, and
the ledger row of one silver unit land in one Postgres transaction
([ADR 0001](../../../docs/adr/0001-mvp-database.md), Amendment 1). Atomicity lives **inside the
adapter**. The ports expose no `begin()`/`commit()` seam, which would leak transactions into every
fake and every engine.

## CQRS (Command Query Responsibility Segregation)

**What:** Separate the write model from the read model when they differ significantly. Writes update
the source of truth; reads can come from views/Redis/Elasticsearch shaped for querying.

**arkad:** Partial (this is `hybrid_data_model.md` §12b: population-vs-query asymmetry). The
ports split by capability: `WriteRepository` writes, `ReadRepository` and `ListRepository` read. The
**raw store is the source of truth**. Silver, graph, and facts are **read-optimized projections**,
and a replay of raw rebuilds them.

## Service Layer

**What:** A layer holding application/business logic (validation, authorization, transactions,
coordinating repositories) so controllers/entry-points stay thin.

**arkad:** The ETL **states** are the service layer. `StoreCompanyFacts` writes the raw document
through a `WriteRepository`. The states depend only on the storage ports.

## Specification

**What:** Represent a business rule / query predicate as a reusable, composable object
(`Adult AND HighIncome AND GoodCredit`) instead of scattered `if` conditions.

**arkad:** Not used. The listing filter of `ListRepository` is the closest: a query object for one
store. If filters must compose, a specification-style API is the path.

## Strategy

**What:** Encapsulate interchangeable algorithms behind one interface; the caller picks one.

**arkad:** Adjacent, not core to storage. The clearest candidates are the deferred **bulk vs
incremental** write facades (two strategies for populating a tier) and concept resolution/mapping in
the SEC adapter. Kept out of the storage ports for now.

## Command

**What:** Represent an operation as an object (`DeleteUserCommand` + a handler), enabling queues,
retries, undo, auditing.

**arkad:** Not used in the storage layer directly. The record passed to
`WriteRepository::persist` is command-like (a self-contained "persist this" message), and the state
machine itself is command-flavored. It matters for a design of retry or backfill queuing.

---

## What arkad's storage layer actually is (the synthesis)

Strip the labels and the storage layer is:

> **Ports & Adapters (hexagonal) at the core**, with a **capability split** — write ports and read
> ports — over a **source-of-truth + projections** storage model, wired by **Dependency Injection**
> and kept honest by the **Clean-Architecture crate boundary**.

Mapping the pieces:

| Concern | Pattern | In arkad |
| --- | --- | --- |
| Decouple from the backend | **Ports & Adapters** | `WriteRepository`/`ReadRepository`/`ListRepository` ↔ `FilesystemRepository`, fakes |
| Enforce the boundary | **Clean Architecture** | `storage` crate (no driver) ← planned `storage-postgres` crate |
| Wire it | **Dependency Injection** | the repository is a generic field of a state context |
| Atomic multi-tier write | **Unit of Work** | planned: one Postgres transaction per silver unit (atomicity inside the adapter) |
| Reads ≠ writes | **Capability split** (CQRS-flavored) | write ports vs read ports; raw = SoT, silver/graph/facts = projections |
| Domain stays SQL-free | **Data Mapper** | `RawDocument`/`FactSet` have no persistence logic |
| Query the read side | **Repository** | `ReadRepository::get`, `ListRepository::list` |

**Why not "just the Repository pattern"?** Repository is an OLTP, entity-centric pattern: a
collection of one aggregate root, read-modify-write, symmetric reads/writes. Our workload is
ETL + analytics + graph: three heterogeneous tiers, write-derived bundles (not mutated aggregates),
and deeply asymmetric reads/writes. Repository fits the *read side* and lends a familiar name; the
*whole* is better described as ports-and-adapters + a capability split over SoT/projections. We reach for the
pattern that fits the use case, and name the parts honestly.

See `design_patterns_demo.py` for a runnable, ~200-line Python model of exactly this combination.
