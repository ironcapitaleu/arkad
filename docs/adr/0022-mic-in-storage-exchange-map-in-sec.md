# ADR 0022: `Mic` Lives in `storage`, the EDGAR Exchange Code Map Lives in `sec`

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-515](https://linear.app/state-machine/issue/STA-515) |

## Context

A listing names its exchange by its MIC, the Market Identifier Code of ISO 10383
([ADR 0009](0009-three-levels-of-identity.md)). `dei:SecurityExchangeName` holds an EDGAR exchange
code, such as `NASDAQ`. Silver keeps the code as filed, so the gold load maps it to a MIC. STA-515
adds a `Mic` type and the map from each EDGAR exchange code to its MIC. The ticket leaves the crate
to the PR.

The choice binds the gold load, each loader of a non-SEC source that names an exchange, and the
`ON_EXCHANGE` edge of a listing. A move later changes the import path of every caller.

Facts on `main` at `b2e400e`, read on 2026-11-15:

- `storage` holds the resolver ports and their key types, `CompanyId` and `ListingId`, under
  `storage::shared` ([ADR 0017](0017-resolver-ports-in-storage.md)).
- `sec` depends on `storage` and on `xbrl`. `xbrl` depends on neither. No other crate depends on
  `storage`.
- The `dei` schema with the enumeration of EDGAR exchange codes is vendored in `xbrl`, at
  `xbrl/taxonomy/dei/2026/dei-2026.xsd`. `Ticker` in `xbrl/src/core/filing.rs` keeps the code as
  `Option<String>`.
- An EDGAR exchange code is an SEC concept. A MIC names an exchange for any source.

## Decision

1. **`Mic` and its error `InvalidMic` live in `storage`, under `storage::shared`,** next to
   `ListingId`. Every loader that names an exchange already reaches `storage`.
2. **The map from an EDGAR exchange code to its MIC lives in `sec`, under `sec::shared`.** It
   returns a `storage::Mic`.

## Options Considered

No option was measured. The facts follow from the `Cargo.toml` files at `b2e400e`.

| Option | A non-SEC loader names a MIC without `sec` | New dependency |
| ------ | ------------------------------------------ | -------------- |
| **A (chosen):** `Mic` in `storage`, map in `sec` | Yes | None |
| B: both in `xbrl`, next to `Ticker` | No. It must depend on `xbrl` | None |
| C: both in `sec` | No. It must depend on `sec` and its HTTP stack | None |
| D: both in `storage` | Yes | None |
| E: `Mic` in `storage`, map in `xbrl` | Yes | `xbrl` on `storage` |

- A puts the key type of an exchange next to the key type of a listing, and the SEC code table in
  the crate of the SEC adapter.
- B and C tie the key of an exchange to one source.
- D puts a table of one source into the crate of the ports. `storage` names no source today.
- E adds a dependency to `xbrl` for one function.

## Consequences

- A caller imports `storage::Mic`. The map is `sec::shared::edgar_exchange_code::mic_of`.
- `storage` holds a third key type that a resolver does not return: a MIC is an identifier of an
  exchange, not a key that the load assigns.
- A loader of another source with its own exchange codes adds its own map in its own crate.

## Open Questions

- The licence of the ISO 10383 list. STA-519 checks the licence of every scheme.

## Sources

- `storage/Cargo.toml`, `sec/Cargo.toml`, `xbrl/Cargo.toml` at `b2e400e`, read on 2026-11-15.
- `xbrl/taxonomy/dei/2026/dei-2026.xsd`, `edgarExchangeCodeItemType`, read on 2026-11-15.
