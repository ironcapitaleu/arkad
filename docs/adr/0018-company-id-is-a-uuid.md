# ADR 0018: `company_id` Is a UUID

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([STA-516 comment `e93ddee0`](https://linear.app/state-machine/issue/STA-516/feature-entityresolver-port-and-its-fake#comment-e93ddee0)). |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-516](https://linear.app/state-machine/issue/STA-516) |

## Context

[ADR 0001](0001-mvp-database.md), "Design Rules" 4, makes `company_id` an internal key that nothing
reuses or rewrites. The Linear document "Entity Resolution: the Company Key, LEI, and Dated CIKs:
Findings" left its format open: "The ID format of `company_id` (UUID or sequence)" is under "Not
checked". STA-516 adds the type `CompanyId`, so the type needs an inner value.

The choice binds every store that holds a `company_id`: the identifier rows, the facts, and the
claims of gold. A change later migrates every stored key.

Facts on `main` at `a9d57d1`, read on 2026-11-15:

- `storage` already depends on `uuid` 1.23 and holds a `Uuid` in `DocumentMetadata`.
- The backfill loads in parallel ([ADR 0004](0004-parallel-backfill.md)).
- Postgres has a 16-byte `uuid` type and an 8-byte `bigint` type.

## Decision

1. **`CompanyId` wraps a `Uuid`.** The value carries no meaning, and no code reads one from it.
2. **The load that assigns a `company_id` picks the UUID version.** `CompanyId` takes any `Uuid`,
   so this decision does not fix the version.

## Options Considered

No option was measured.

| Option | Who assigns a new key | Size in Postgres | Shows the order or the count of companies |
| ------ | --------------------- | ---------------- | ----------------------------------------- |
| **A (chosen):** UUID | Any process, with no shared state | 16 bytes | No, for a random UUID |
| B: sequence (`bigint`) | One counter in the database | 8 bytes | Yes |

- A lets a parallel load or a test assign a key with no round trip to the database. It needs no
  new dependency.
- B halves the key size, but every new key needs the counter of one database. A test or a second
  store cannot assign a key that the database later accepts.

## Consequences

- A key is 16 bytes in every row that holds it.
- A key in a log or an API response is a 36-character string. The API returns the LEI, the CIK,
  and the ticker next to it, as the findings document says.
- No code can derive a CIK or an LEI from a key, so no code can treat the key as an identifier.

## Open Questions

- The UUID version: a random UUID (version 4), or a time-ordered UUID (version 7) for index
  locality in Postgres. The ticket of the gold load decides it.

## Sources

- Linear document "Entity Resolution: the Company Key, LEI, and Dated CIKs: Findings", "Open
  Points", read on 2026-11-15.
- `storage/Cargo.toml` and
  `storage/src/implementations/repository/filesystem/shared/document_metadata/mod.rs` on `main` at
  `a9d57d1`.
- [PostgreSQL 17 documentation, "UUID Type" and "Numeric Types"](https://www.postgresql.org/docs/17/datatype-uuid.html).
- ADR 0001, "Design Rules" 4. ADR 0004.
