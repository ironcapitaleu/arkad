# ADR 0006: Storage Errors Mirror the `sec` Error Hierarchy

| Field | Value |
| ----- | ----- |
| Status | Accepted by the lead on 2026-11-14 ([STA-512 comment](https://linear.app/state-machine/issue/STA-512/docs-replace-the-storage-trait-design-with-an-overview-and-adrs#comment-e112485d)). |
| Date | 2026-08-10 |
| Deciders | The project lead |
| Ticket | [STA-145](https://linear.app/state-machine/issue/STA-145) made the decision. [STA-512](https://linear.app/state-machine/issue/STA-512) records it. |

## Context

Every method of a `storage` port returns an error type of `storage::error`. Each backend builds
these errors, and `sec` reads them. A change to their shape changes the public interface of
`storage` and each backend.

The `sec` crate has an error hierarchy of its own (`sec/src/lib/error/`): one module per level, an
`ErrorKind` at the top, `From` from a lower level to a higher level, `TryFrom` back down, and a
`DowncastNotPossible` variant for a `TryFrom` that does not match. Each level derives `Clone`,
`PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash`. A type with these derives is a **value type**
in this ADR.

A value type can hold only value types. `Box<dyn std::error::Error>` is no value type, and neither
is `sqlx::Error`.

## Decision

1. **Every storage error is a value type.** No storage error holds a `Box<dyn Error>`. A backend
   turns an error that is no value type, such as `sqlx::Error`, into a reason string at its own
   boundary.
2. **The storage errors use the levels and conversions of the `sec` hierarchy.** One module per
   level, `From` up, `TryFrom` down, and `ErrorKind::DowncastNotPossible` for a `TryFrom` that does
   not match.
3. **A method returns the error of its operation.** A read method returns `ReadError`. A write
   method returns `WriteError`. Both wrap the shared `BackendError`. `ErrorKind` wraps
   `ReadError` and `WriteError`, for a caller that handles both.
4. **The type names end in `Error`.** `ReadError`, `WriteError`, and `BackendError` keep the suffix,
   unlike `State` and `Transition` in `sec`. A bare `Read` or `Write` collides with
   `std::io::Read`, `std::io::Write`, and `std::fmt::Write` in a `use`. The variants stay short,
   such as `WriteError::Backend`.

## Options Considered

No option was measured. The facts below follow from the Rust trait rules and the derives of `sec`.

| Option | `assert_eq!` on the error | An `sec` error can hold it | Keeps the backend error as an object |
| ------ | ------------------------- | -------------------------- | ------------------------------------ |
| **A (chosen):** value types in the `sec` shape | Yes | Yes | Only if the backend error is a value type |
| B: one error type with a boxed `source()` | No, `dyn Error` has no `PartialEq` | No | Yes |
| C: an associated `type Error` per backend | Only if each backend error is a value type | Only for a known backend | Yes |

- A lets each test compare the error with `assert_eq!`, as `AGENTS.md`, "Testing", requires. A
  value type of `sec` can hold it.
- B loses `Clone`, `Eq`, and `Hash`, so no `sec` error type can derive them and hold it.
- C makes each caller generic over the error. A non-generic `sec` error type cannot hold
  `R::Error`. Decision 1 already turns each backend error into a value type, so an associated type
  adds no information.

## Consequences

- A test asserts on the whole error value with one `assert_eq!`.
- A `sqlx::Error` reaches the log as text only. A caller cannot match on its type.
- A backend error that is a value type can stay an object. `BackendCause` holds a reason string or
  a `FailedFilesystemOperation`.
- `ReadError` cannot carry `ConflictingWrite`, and `WriteError` cannot carry `MissingRecord`.
- `sec` turns a storage error into an error of its own at the call that returns it
  (`sec/src/lib/shared/document_storage/`). It holds the storage error as text, so `source()` ends
  there.

## Sources

- `sec/design/data_model/storage_traits_design.md` at commit `9cf103c`
  (ironcapitaleu/arkad#133, merged on 2026-08-11), "Error model — mirrors the `sec` error
  hierarchy". Its revision note dates the value type and the `sec` shape to 2026-08-10.
- `storage/src/error/` and `sec/src/lib/error/` on `main` at `070b916`, read on 2026-11-14.
- `sec/src/lib/shared/document_storage/mod.rs` on `main` at `070b916`, read on 2026-11-14.
