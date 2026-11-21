# ADR 0031: `AccountId` Lives in `storage`, and `api` Imports It

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-21 ([STA-311 comment `0d86c043`](https://linear.app/state-machine/issue/STA-311#comment-0d86c043)) |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-311](https://linear.app/state-machine/issue/STA-311) |

## Context

An account is one Supabase user, keyed by the user ID (`auth.users.id`), a UUID. A login token
carries the ID in its `sub` claim. STA-311 adds the `TokenVerifier` port in `api`, which turns a
login token into an `AccountId`. The ticket asks for `AccountId` as a domain type that rejects a
value that is not a UUID. It does not fix the crate of the type.

The choice binds every later port that keys on an account, and the crate that each caller imports
the type from. A move later changes the import path of every caller.

Facts on `main` at `f550792`, read on 2026-11-15:

- The lead decided on 2026-10-25 that the persistence ports and their Postgres backend live in
  `storage`, and that `TokenVerifier` lives in `api`. The Linear document "Access Control for
  `/v1`: Tokens, API Keys, and Quotas (Draft)" records the decision in its table "Inputs".
- The same Linear document plans `storage` ports that key on an account:
  `ApiKeyRepository::list_by_account(&AccountId)`, `ApiKeyRepository::revoke(&AccountId, …)`,
  `AccountPlanRepository` as `ReadRepository<Key = AccountId>`, and `QuotaRequest` with an
  `account_id` field.
- `storage` depends on no crate of the workspace. `api` depended on `xbrl` only, and `sec` depends on
  `storage` and `xbrl`.
- `storage::shared` holds the key types of the resolver ports, `CompanyId` and `ListingId`
  ([ADR 0017](0017-resolver-ports-in-storage.md)).

## Decision

1. **`AccountId` and its error `InvalidAccountId` live in `storage`, under `storage::shared`,**
   next to `CompanyId`. `AccountId::parse` rejects a string that is not a UUID.
2. **`api` depends on `storage` and imports `storage::AccountId`.** The `TokenVerifier` port, its
   `VerifiedToken`, and its `TokenError` stay in `api`.

## Options Considered

No option was measured. The facts follow from the `Cargo.toml` files at `f550792`.

| Option | A `storage` port can name the type | New dependency |
| ------ | ---------------------------------- | -------------- |
| **A (chosen):** `AccountId` in `storage` | Yes | `api` on `storage` |
| B: `AccountId` in `api` | No. `storage` must not depend on `api`, because `api` calls the `storage` ports | None |
| C: a new crate of shared domain types | Yes | `api` and `storage` on the new crate |

- A puts the key of an account next to the key of a company, where the ports that key on it live.
- B works for STA-311 alone. The first `storage` port that keys on an account then moves the type
  to `storage` and changes every import.
- C adds a crate for one type. `storage::shared` already serves as the place for key types.

## Consequences

- `api` depends on `storage`. `storage` pulls in no database driver ([ADR 0005](0005-storage-ports-crate.md)),
  and `cargo tree -p api -i openssl-sys` finds no package on 2026-11-15.
- A later port in `storage`, such as `ApiKeyRepository` or `QuotaStore`, names `AccountId` without
  a new dependency.
- `AccountId` takes any UUID version. Supabase picks the version of a user ID.
- The decision does not cover the other types of the Linear document "Access Control for `/v1`",
  such as `ApiKeyId` or `UsageKey`. The ticket of each port decides where its types live.

## Open Questions

- Whether a user with two emails, one through Google and one through GitHub, keeps one account. A
  comment of the lead on the Linear document asks how social logins give one view of a user. The
  answer changes what an `AccountId` names, not its type or its crate.

## Sources

- `api/Cargo.toml`, `storage/Cargo.toml`, and `sec/Cargo.toml` at `f550792`, read on 2026-11-15.
- Linear document "Access Control for `/v1`: Tokens, API Keys, and Quotas (Draft)", sections
  "Inputs" and "Traits", read on 2026-11-15.
- [STA-311](https://linear.app/state-machine/issue/STA-311), Definition of Done, read on
  2026-11-15.
