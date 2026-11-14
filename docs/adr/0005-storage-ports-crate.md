# ADR 0005: No Database Driver in the `storage` Crate

| Field | Value |
| ----- | ----- |
| Status | Accepted by the lead on 2026-11-14 ([STA-512 comment](https://linear.app/state-machine/issue/STA-512/docs-replace-the-storage-trait-design-with-an-overview-and-adrs#comment-e112485d)). |
| Date | 2026-08-08 |
| Deciders | The project lead |
| Ticket | [STA-130](https://linear.app/state-machine/issue/STA-130) made the decision. [STA-512](https://linear.app/state-machine/issue/STA-512) records it. |

## Context

The `storage` crate holds the storage ports (`WriteRepository`, `ReadRepository`,
`ListRepository`, and their marker traits), the error types, and the filesystem backend
`FilesystemRepository`. The `sec` crate depends on `storage`. Every binary of `sec` depends on it
through `sec`.

[ADR 0001](0001-mvp-database.md) puts the graph tier and the fact tier in Postgres. Its Amendment 1
puts the load of a silver unit into impl-private helpers "inside `storage-postgres`". A Postgres
backend needs a driver, such as `sqlx` or `tokio-postgres` (ADR 0001, "Options Considered").

Cargo compiles every non-optional dependency of a crate for each crate that depends on it. If a
driver sits in `storage`, `sec` and each of its binaries compile the driver, and `sec` code can name
a Postgres type.

The choice binds every crate that adds a backend, and the load step of ADR 0001. A reversal moves
public types from one crate to another.

Terms:

- **Driver**: a crate that speaks the wire protocol of a database, such as `sqlx`.
- **Composition root**: the `main` of a binary, where the concrete backends are built and passed in.

## Decision

1. **`storage` depends on no database driver.** It holds the ports, the error types, and the
   backends whose dependencies are no driver.
2. **The Postgres backend lives in a crate `storage-postgres`.** `storage-postgres` depends on
   `storage`. `storage` never depends on `storage-postgres`.
3. **Only a composition root names `storage-postgres`.** Library code, such as `sec`, takes a type
   that implements a `storage` port.

## Options Considered

No option was measured. The facts below follow from how Cargo resolves dependencies and features.

| Option | `sec` compiles the driver | `sec` code can name a Postgres type | Crates |
| ------ | ------------------------- | ----------------------------------- | ------ |
| **A (chosen):** a crate `storage-postgres` | No | No | One more |
| B: a module in `storage` behind a Cargo feature | Yes, if one crate of the build turns the feature on | Yes, if the feature is on | No change |
| C: a module in `storage` | Yes | Yes | No change |

- A keeps the driver out of the dependency graph of `sec`, so the compiler rejects a Postgres type
  in `sec`.
- B depends on the feature set of the build. Cargo unifies the features of one crate across a
  build, so one binary that turns the feature on turns it on for `sec` in that build.
- C adds the driver to every crate that depends on `storage`.

## Consequences

- A build of `sec` needs no Postgres driver and no database.
- `storage-postgres` sees only the public items of `storage`. A helper that it needs from `storage`
  must be public.
- `FilesystemRepository` stays in `storage`. Its file I/O goes through `tokio` with the `fs`
  feature, which is no driver. It entered `storage` in ironcapitaleu/arkad#204 on 2026-10-04.
- A further backend with a driver, such as Neo4j behind the graph port of ADR 0001, gets its own
  crate by the same rule.
- The workspace has no `storage-postgres` crate yet. The load step of ADR 0001 adds it.

## Open Questions

- Whether `FilesystemRepository` moves to a crate of its own. Decision 1 does not require it. The
  lead decides.

## Sources

- `sec/design/data_model/storage_traits_design.md` at commit `0212e6e`
  (ironcapitaleu/arkad#124, merged on 2026-08-10), "Crate topology (from the `storage`-crate
  decision)". Its revision note dates the crate decision and its name to 2026-08-08.
- `storage/Cargo.toml` and `sec/Cargo.toml` on `main` at `070b916`, read on 2026-11-14: the
  dependencies of `storage` hold no driver, and `sec` depends on `storage`.
- [Cargo Book, "Features", "Feature unification"](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification).
