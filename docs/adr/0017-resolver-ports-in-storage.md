# ADR 0017: The Resolver Ports Live in the `storage` Crate

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([STA-516 comment `e93ddee0`](https://linear.app/state-machine/issue/STA-516/feature-entityresolver-port-and-its-fake#comment-e93ddee0)). |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-516](https://linear.app/state-machine/issue/STA-516) |

## Context

[ADR 0001](0001-mvp-database.md), "Design Rules" 5, names one resolver port that hides the engine.
[ADR 0009](0009-three-levels-of-identity.md) gives each level of identity its own resolver port.
STA-516 adds `EntityResolver`, the port of the company level, with its key type `CompanyId`. The
ticket leaves the crate of the port to the PR.

The choice binds `ListingResolver` (STA-517), a resolver of the security level, the SEC adapter
that resolves a filing by its CIK, and the gold load. A move later changes the import path of
every caller and every adapter.

Facts on `main` at `a9d57d1`, read on 2026-11-15:

- `storage` holds the storage ports and depends on no database driver
  ([ADR 0005](0005-storage-ports-crate.md)). Its dependencies already include `async-trait`,
  `chrono`, and `uuid`.
- `sec` depends on `storage`. No other crate depends on `storage` or on `sec`.
- `Cik` lives in `sec/src/lib/shared/cik/`. The port does not need it: a resolver matches the
  value of an identifier as text, with exact keys only (ADR 0001, "Design Rules" 3).
- A resolver reads dated identifier rows from a store. A real resolver is a read of the metadata
  store of gold (ADR 0007) or of the CIK range table.

## Decision

1. **The resolver ports and their key types live in `storage`,** under `storage::traits::resolver`
   and `storage::shared`. `EntityResolver` and `CompanyId` are the first. `ListingResolver` and
   `ListingId` follow in the same places.
2. **An adapter that needs a driver lives in its own crate,** as ADR 0005, Decision 2, says for a
   storage backend.

## Options Considered

No option was measured. The facts follow from the `Cargo.toml` files at `a9d57d1`.

| Option | A non-SEC source can use the port without `sec` | The SEC adapter can use the port | New crates |
| ------ | ----------------------------------------------- | -------------------------------- | ---------- |
| **A (chosen):** `storage` | Yes | Yes, `sec` depends on `storage` | None |
| B: `sec`, next to `Cik` | No. It must depend on `sec` and its HTTP stack | Yes | None |
| C: a new crate, such as `identity` | Yes | Yes, after `sec` adds the dependency | One |

- A puts the port next to the other read ports, in a crate that each loader already reaches.
- B ties the company level to one source. A loader of a non-SEC source, such as DART, compiles the
  SEC client to name a `CompanyId`.
- C keeps the resolvers apart from the repositories, but adds a crate and a dependency for two
  traits and two key types.

## Consequences

- Every caller imports `storage::{CompanyId, EntityResolver}`. The crate needs no new dependency.
- `storage` holds two kinds of port: repositories that read and write records by key, and
  resolvers that map an identifier to a key.
- The fakes of the resolvers live in `storage/src/tests/fixtures/`, compiled for the tests of
  `storage` only, as the other fakes of `storage` are ([ADR 0016](0016-ports-and-fakes.md),
  Decision 3).
- This ADR does not decide the key format. [ADR 0018](0018-company-id-is-a-uuid.md) holds it.

## Open Questions

- Whether a crate outside `storage` needs the resolver fakes in its own tests. If the gold load
  does, `storage` can expose them behind a feature, as `sec` does with `test-fixtures`. The ticket
  of the gold load decides it.
- Whether the resolvers move to a crate of their own once there are more than the three of ADR
  0009. The project lead decides it.

## Sources

- `storage/Cargo.toml` and `sec/Cargo.toml` on `main` at `a9d57d1`, read on 2026-11-15.
- [STA-516](https://linear.app/state-machine/issue/STA-516), "Additional Notes", read on
  2026-11-15.
- ADR 0001, "Design Rules" 3 and 5. ADR 0005, Decisions 1 and 2. ADR 0009, "Consequences".
