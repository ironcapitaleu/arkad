# Capability-Split Persistence Ports — Findings (STA-157)

## Purpose

This document records the SPIKE findings for splitting the `storage` persistence ports by access
capability. The goal is a design where the type system enforces read versus write access. A caller
handed a read-only port cannot call a write method, and the mistake is a compile error.

## Context

The `storage` crate exposes one write-only port today:

```rust
#[async_trait]
pub trait Repository: Send + Sync {
    type Record: Send;
    async fn persist(&self, record: Self::Record) -> Result<(), WriteError>;
}
```

The error side carries a single operation class so far. `ErrorKind` wraps `Write(WriteError)` and
recovers the specific class through `TryFrom`. `WriteError` wraps a leaf `BackendError`.

The `feat/queue` branch already encodes access in the type system. Its `ChannelBuilder` yields a
`ProducerChannel` with a `send` method (write) or a `ConsumerChannel` with a `receive` method
(read). One handle carries one capability. Calling the wrong side does not compile. This SPIKE
brings that same split to the `storage` ports, so one pattern covers both the database repository
and the future queue-backed repository.

## The Two Approaches

### Approach A — Capability Traits

Split the port into one trait per capability. A component depends on the trait it needs.

```rust
#[async_trait]
pub trait ReadRepository: Send + Sync {
    type Record: Send;
    type Key: Send;
    async fn get(&self, key: Self::Key) -> Result<Option<Self::Record>, ReadError>;
}

#[async_trait]
pub trait WriteRepository: Send + Sync {
    type Record: Send;
    async fn persist(&self, record: Self::Record) -> Result<(), WriteError>;
}

/// A store that offers both capabilities.
pub trait ReadWriteRepository: ReadRepository + WriteRepository {}
impl<T: ReadRepository + WriteRepository> ReadWriteRepository for T {}
```

A state that only reads takes a `R: ReadRepository`. It has no `persist` method in scope, so a
write is unrepresentable. A backend adapter implements both traits, and the blanket impl gives it
`ReadWriteRepository` for free.

**Pros:**

- Access is the trait bound. The read-only caller has no write method.
- Object-safe. `Box<dyn ReadRepository>` and `Arc<dyn WriteRepository>` both work with
  `async_trait`, so `dyn` dependency injection keeps working as it does today.
- Minimal. The current `Repository` becomes `WriteRepository` with a rename. The read side is a new
  sibling trait, not a rework.
- Composes with the existing DI. Each `State` binds an associated type to the capability it needs.

**Cons:**

- Two traits to implement for a full backend, though the blanket impl removes the third.
- The capability lives in the bound, not in a single named handle. A caller that wants "a read
  handle" as one value uses `impl ReadRepository` or `Box<dyn ReadRepository>`.

### Approach B — Type-State Marker

Keep one `Repository` type and gate its methods with a phantom capability marker. This mirrors the
`ChannelBuilder` markers directly.

```rust
pub struct Read;
pub struct Write;
pub struct ReadWrite;

pub struct Repository<Backend, Cap> {
    backend: Backend,
    _capability: PhantomData<Cap>,
}

impl<B: Backend> Repository<B, Read> {
    async fn get(&self, key: B::Key) -> Result<Option<B::Record>, ReadError> { /* ... */ }
}

impl<B: Backend> Repository<B, Write> {
    async fn persist(&self, record: B::Record) -> Result<(), WriteError> { /* ... */ }
}
```

A `Repository<Pg, Read>` has no `persist` in scope. The capability is chosen at construction, the
same way `.producer()` and `.consumer()` choose the channel type.

**Pros:**

- One named handle carries its capability, which suits construction by a builder.
- The `ReadWrite` marker can expose both method sets through one more `impl` block.

**Cons:**

- Not object-safe in the useful direction. `dyn` erases the marker, so `Box<dyn ...>` cannot carry
  the capability. The current DI passes trait objects, so this approach works against that wiring.
- The capability is a concrete-type parameter, not a trait bound. A generic consumer that wants
  "anything readable" still needs a trait, so Approach B tends to grow an Approach-A trait anyway.
- Heavier. Every method sits behind a marker-bounded `impl` block.

## Error Hierarchy Fit

The read side needs its own leaf error, mirroring `WriteError`. The naming follows the
Adjective-First and Failed-First conventions in `AGENTS.md`.

```rust
#[non_exhaustive]
pub enum ReadError {
    MissingRecord(/* ... */),   // the key resolves to nothing when a record is required
    FailedDeserialization(/* ... */),
    Backend(BackendError),      // same leaf backend error the write side already wraps
}
```

`ErrorKind` gains a second variant next to `Write`, and the same `TryFrom` downcast pattern
extends to it:

```rust
#[non_exhaustive]
pub enum ErrorKind {
    Read(#[source] ReadError),
    Write(#[source] WriteError),
    DowncastNotPossible,
}
```

`BackendError` stays the shared leaf. Both `ReadError::Backend` and `WriteError::Backend` wrap it,
so the skip-level downcast to `BackendError` keeps working from either side. This reuses the
hierarchy STA-139 and STA-147 already built, rather than duplicating it.

## Mapping the Queue Split

The queue's Producer/Consumer split maps onto the ports one-to-one:

| Queue (`feat/queue`) | Storage port | Capability |
| --- | --- | --- |
| `ProducerChannel::send` | `WriteRepository::persist` | Write / outbound |
| `ConsumerChannel::receive` | `ReadRepository::get` | Read / inbound |
| `ChannelBuilder` markers | `ReadWrite` supertrait bound | Both |

So when the queue lands as a persistence port, its Producer becomes a `WriteRepository` and its
Consumer becomes a `ReadRepository`. The `inner` lapin channel becomes the backend adapter. One
capability model then covers the database store and the queue-backed store.

## Testing Implications

The current `FakeRepository<Rec>` records every persisted record and returns `Ok`. Under Approach
A it splits by capability:

- `FakeReadRepository<Rec>` — seeded with records, serves `get`, and can be told to return
  `ReadError` cases for the failure tests.
- The existing `FakeRepository<Rec>` becomes the `WriteRepository` fake unchanged.
- One `FakeStore` can implement both traits when a test needs a round trip (write then read back).

Each capability gets its own fake, so a read-only consumer test wires only the read fake. This
keeps a consumer test pinned to the one trait it depends on.

## Limitations

- A component that both reads and writes takes two bounds (`R: ReadRepository + WriteRepository`)
  or the `ReadWriteRepository` alias. This is the correct cost, because the type now states the
  component needs both.
- Capability at the type level guards the read and write API, not the database grants. A `ReadRepository`
  adapter must still connect with least-privilege credentials. The type system cannot enforce the
  backend's own permissions.
- Transactions and multi-record units are out of scope for this SPIKE. The split does not model
  them, and a later ticket must decide where a transaction boundary sits.

## Recommendation

Use **Approach A (capability traits)** for the ports:

1. Rename the current `Repository` to `WriteRepository`, keeping `persist` and `WriteError`.
2. Add a sibling `ReadRepository` with `get` and a new `ReadError`.
3. Add the `ReadWriteRepository` supertrait plus its blanket impl for stores that offer both.
4. Add `ErrorKind::Read(ReadError)` and its `From` / `TryFrom` conversions, reusing `BackendError`
   as the shared leaf.
5. Split the fakes by capability.

Reserve **Approach B (type-state marker)** for a later builder that constructs a concrete handle,
if a builder is wanted. This is the same role `ChannelBuilder` plays for the queue. The ports
themselves stay trait-based, because the DI passes trait objects and Approach A stays object-safe.

## Next Steps

A follow-up FEATURE ticket implements the read side per this recommendation: the `ReadRepository`
trait, the `ReadError` type and its `ErrorKind` wiring, the read fake, and the full test suite
(conversions, `Display`, and the capability separation).
