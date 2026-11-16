# ADR 0036: The GCS Backend of Silver Sits in `storage` Behind a Cargo Feature

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project owner confirms |
| Date | 2026-11-16 |
| Deciders | The project owner |
| Ticket | [STA-463](https://linear.app/state-machine/issue/STA-463/design-silver-writer-and-transform-job-plan) |

## Context

Silver writes Parquet parts and lease objects to GCS. It needs two preconditions of the GCS JSON
API: `ifGenerationMatch=0`, to create an object only if it is absent, and
`ifGenerationMatch=<generation>`, to replace a lease only if nobody changed it. The bucket mount
shows no generation, so silver needs a client of the JSON API.

[ADR 0005](0005-storage-ports-crate.md) puts the Postgres backend in a crate of its own. Its reason
is feature unification: Cargo unifies the features of one crate across a build, so if one binary turns
a feature of `storage` on, the feature is on for `storage` in the whole build. ADR 0005 covers backends
with a database driver. A GCS client of the JSON API is no database driver, so ADR 0005 does not
decide this case.

Facts that bound the choice:

- The `Dockerfile` builds `backfill` and `daily_filings` of `sec` in one `cargo zigbuild` call.
  The silver binaries `transform` and `series` will sit in `sec` and join that call.
- `sec` already depends on `reqwest` 0.13 for the SEC client. Every `sec` binary compiles it.
- The JSON API takes plain HTTPS requests. On Cloud Run, the token comes from the metadata server,
  also over HTTP.

The owner chose a cargo feature over a new crate on 2026-11-16, in the chat of this design.

## Decision

1. **The GCS backend of `SilverObjectStore` and `LeaseStore` sits in `storage`, behind the cargo
   feature `gcs`.**
2. **The backend sends its requests through `reqwest`,** the client that `sec` already compiles.
   It adds no GCS SDK.

## Options Considered

Compared on 2026-11-16, on `main` at commit `1ab9d97`.

| Option | Crates | `backfill` and `daily_filings` compile the backend | New dependencies |
| ------ | ------ | -------------------------------------------------- | ---------------- |
| **A (chosen). A module in `storage` behind the feature `gcs`** | No change | Yes, if one binary of the build turns the feature on | A CRC32C crate, on top of `reqwest` |
| B. A crate `storage-gcs`, as ADR 0005 does for Postgres | One more | No, if the silver binaries sit in their own package. Yes, if they sit in `sec`, because the build compiles every binary of `sec` with its dependencies | As A |

- A and B give the same binaries while the silver binaries sit in `sec`. Only B with a separate
  package keeps the backend out of the extract binaries.
- A adds no crate. Its extra code in each binary is the backend and a CRC32C crate, because
  `reqwest` is already there.

## Consequences

- Every binary of a build that turns on `gcs` compiles the backend, the extract binaries included.
- The backend uses `rustls` through `reqwest`. `cargo tree -i openssl-sys` must still find no
  package.
- A backend that needs a large SDK or a database driver gets a crate of its own, as ADR 0005 says.

## Open Questions

- If the backend needs an SDK, for example for a token outside Cloud Run, its size reopens this
  decision.
- If the bucket mount honors "create only if absent", silver can write parts through the mount,
  and only the lease needs the client.

## Sources

- [ADR 0005](0005-storage-ports-crate.md), accepted: the reason of feature unification.
- `Dockerfile` and `sec/Cargo.toml` on `main` at commit `1ab9d97`: one build call for the `sec`
  binaries, and `reqwest` 0.13 as a dependency of `sec`.
- [Cargo Book, "Features", "Feature unification"](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification).
