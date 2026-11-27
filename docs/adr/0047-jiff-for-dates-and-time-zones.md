# ADR 0047: `jiff` for Dates, Timestamps, and Time Zones

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-27 ([STA-592 comment `cde2b58a`](https://linear.app/state-machine/issue/STA-592#comment-cde2b58a), accessed 2026-11-27). Amendment 1: accepted by the project owner on 2026-11-27 ([STA-593 comment `91a3878f`](https://linear.app/state-machine/issue/STA-593#comment-91a3878f), accessed 2026-11-27) |
| Date | 2026-11-24 |
| Deciders | The project owner |
| Ticket | [STA-591](https://linear.app/state-machine/issue/STA-591), accessed 2026-11-27. First slice [STA-592](https://linear.app/state-machine/issue/STA-592), accessed 2026-11-27 |

## Context

The crates `storage`, `xbrl`, `api`, and `sec` use `chrono` for dates and timestamps. `xbrl` and
`sec` use `chrono-tz` for the New York time zone. On 2026-11-24, `cargo audit` on `main` reported
both crates as unmaintained:

- `chrono` 0.4.45: RUSTSEC-2026-0543.
- `chrono-tz` 0.10.4: RUSTSEC-2026-0555.

No third-party crate in `Cargo.lock` depends on `chrono`, so both crates can leave the tree.

Three places need the New York time zone:

- `sec/src/bin/shared/new_york_clock.rs` and `sec/src/bin/daily_filings/dates.rs` read today in New
  York. The SEC's days are Eastern.
- `xbrl/src/adapter/edgar/notes/row.rs` turns `sub.accepted`, a New York time, into UTC.

The binaries are static and run on `gcr.io/distroless/static-debian13`.
[ADR 0002](0002-static-binaries.md) requires that a binary compiles in the data it needs and reads
no zone file of the image.

Public types pass `chrono` dates across crate boundaries. The move goes crate by crate, so `chrono`
and `jiff` both stay in the tree until the last slice merges.

## Decision

1. **Every crate uses `jiff` for dates, timestamps, and time zones.** `chrono` and `chrono-tz`
   leave every `Cargo.toml`.
2. **`jiff` builds without its default features.** The defaults turn on `tz-system`,
   `tzdb-zoneinfo`, and `tzdb-concatenated`, which read zone files of the system at run time.
   Without them, `jiff` reads no zone file.
3. **A time zone comes from the macro `jiff::tz::get!`, with the feature `static`.** The macro
   compiles one zone into the binary at build time, from the tz database of the crate `jiff-tzdb`.
   `jiff-tzdb` is a dependency of the proc macro only, so the binary holds no other zone. A wrong
   zone name fails the build, not the run.
4. **Every stored date and timestamp string stays byte for byte the same.** Each slice that changes
   a serialized type adds a round-trip test. The test reads a string that `chrono` wrote and writes
   the same string back. A slice that cannot keep a string stops, because a format change needs a
   migration and its own ADR.

The features of the first slice are `std` and `static`. A later slice adds `serde` when it moves a
serialized type.

## Amendment 1: Fraction Digits of a Stored Timestamp

Date: 2026-11-27. Status: Accepted by the project owner on 2026-11-27.
Ticket: [STA-593](https://linear.app/state-machine/issue/STA-593), accessed 2026-11-27.

### Reason

Decision 4 keeps every stored string. The open question asked how many fraction digits a stored
timestamp has. Two writers differ:

- `chrono` serde writes a `DateTime<Utc>` with `SecondsFormat::AutoSi` and `Z`: no fraction for a
  whole second, else 3, 6, or 9 digits. Source: `chrono` 0.4.45, `src/datetime/serde.rs`.
- `jiff` 1.0.3 writes a `Timestamp` with the fewest digits. On 2026-11-27 it printed
  `2026-09-22T20:05:30.5Z`, where `chrono` writes `2026-09-22T20:05:30.500Z`.

`storage` stores two timestamps: `produced_at` in the metadata of each document, and `expires_at`
in each lease. A run sets both from the clock, so most stored values have a fraction. The plain
`jiff` serde would change these strings.

### Decision

1. **A stored timestamp keeps the form that `chrono` writes.** The form is RFC 3339 in UTC with
   `Z`. The fraction has no digits for a whole second, else 3, 6, or 9 digits: the fewest of these
   that hold the nanoseconds.
2. **`storage` writes and reads the form with its own serde functions.** Each stored `Timestamp`
   field names them with `#[serde(with = ...)]`. They read any RFC 3339 timestamp with an offset,
   as `chrono` does.
3. **`storage` builds `jiff` without the feature `serde`.** It stores no `jiff` value through the
   plain `jiff` serde.

Options considered:

| Option | Stored strings | Cost |
| ------ | -------------- | ---- |
| **A (chosen)**: serde functions in `storage` | Same bytes | One module of about 50 lines |
| B: plain `jiff` serde | `.500Z` becomes `.5Z`, `.123400Z` becomes `.1234Z` | A migration and a new ADR |

### Consequences

- Round-trip tests read the JSON that `chrono` wrote and compare the written bytes. They cover no
  fraction, 3, 6, and 9 digits, for both the metadata and the lease.
- A later slice that stores a `jiff` timestamp outside `storage` checks its own form against
  decision 4. This amendment covers the fields of `storage` only.
- `storage` stores no date. The form of a stored `jiff::civil::Date` stays open for the slices
  that store one.

## Options Considered

Measured on 2026-11-24 on `main` at `850a823`, with jiff 1.0.3 and jiff-tzdb 1.0.0 (tzdb
`2026f`). Each build ran
`cargo zigbuild --release --locked -p sec --features tracing-logging --bin backfill --bin daily_filings --target x86_64-unknown-linux-musl`.
`readelf -d` listed no shared library for any build.

| Option | Zone source | Zone lookup | `backfill` | `daily_filings` |
| ------ | ----------- | ----------- | ---------- | --------------- |
| `main` today | `chrono-tz`, all zones compiled in | compile time | 7,749,128 B | 7,585,912 B |
| **A (chosen)** | `jiff` `static`, one zone compiled in | compile time | 7,985,944 B | 7,822,152 B |
| B | `jiff` `tzdb-bundle-always`, all zones compiled in | run time, by name | 8,227,576 B | 8,063,656 B |
| C | `jiff` default features | run time, from `/usr/share/zoneinfo` first | not built | not built |

- A against B: A is about 242 KB smaller per binary, and a wrong zone name fails the build. B looks up
  the zone at run time, and the lookup returns a `Result` that the code must handle.
- C breaks ADR 0002. It reads the zone files of the image when they exist, so the result depends on
  the image.
- A makes each binary about 237 KB larger than `main` today, and B about 478 KB. The measure is
  the code of `jiff` against the code of `chrono` and `chrono-tz`, while both crates stay in the
  tree. The last slice removes `chrono`.
- `time` was not considered: it has no IANA time zones.

## Consequences

- `chrono-tz` leaves `Cargo.lock` with the first slice. `chrono` leaves with the last slice.
- A new time zone needs one more `tz::get!` call. No feature changes.
- The zone data updates only when `jiff-tzdb` updates. A rule change of New York needs a
  `cargo update -p jiff-tzdb` and a new build.
- A `jiff` value that reads the system zone, such as `ZonedDateTime::now()`, gets the zone
  `Etc/Unknown`, because `tz-system` is off. That zone has the offset of UTC. The code must name
  the zone with `tz::get!`.
- Until the last slice merges, the code converts between `chrono` and `jiff` values at each place
  that has moved. Each slice removes the conversions of its crate.

## Open Questions

- `chrono` writes the fraction of a second with 3, 6, or 9 digits. `jiff` writes the fewest digits.
  The `storage` slice ([STA-593](https://linear.app/state-machine/issue/STA-593), accessed 2026-11-27) checks whether a
  stored timestamp has a fraction, and decision 4 holds it to the same string.

## Sources

- `cargo audit` on `main` at `850a823`, 2026-11-24: RUSTSEC-2026-0543 (`chrono`) and
  RUSTSEC-2026-0555 (`chrono-tz`).
- `cargo info jiff`, 2026-11-24: version 1.0.3 and its default features.
- `jiff::ZonedDateTime::now()` with `jiff` 1.0.3 and the features `std` and `static`, 2026-11-24:
  printed `2026-11-24T16:08:36.749349896Z[Etc/Unknown]`.
- `jiff-tzdb` 1.0.0, `tzname.rs`: `VERSION = "2026f"`.
- The builds and sizes in "Options Considered", 2026-11-24.
- [ADR 0002](0002-static-binaries.md), "Consequences": a binary compiles in the data it needs.
