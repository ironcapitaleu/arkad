# ADR 0044: `jiff` for Dates, Timestamps, and Time Zones

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project owner confirms |
| Date | 2026-11-24 |
| Deciders | The project owner |
| Ticket | [STA-591](https://linear.app/state-machine/issue/STA-591), first slice [STA-592](https://linear.app/state-machine/issue/STA-592) |

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

- A against B: A is 241,632 B smaller per binary, and a wrong zone name fails the build. B looks up
  the zone at run time, and the lookup returns a `Result` that the code must handle.
- C breaks ADR 0002. It reads the zone files of the image when they exist, so the result depends on
  the image.
- A and B both make each binary about 237 KB larger than `main` today. The measure is the code of
  `jiff` against the code of `chrono` and `chrono-tz`, while both crates stay in the tree. The last
  slice removes `chrono`.
- `time` was not considered: it has no IANA time zones.

## Consequences

- `chrono-tz` leaves `Cargo.lock` with the first slice. `chrono` leaves with the last slice.
- A new time zone needs one more `tz::get!` call. No feature changes.
- The zone data updates only when `jiff-tzdb` updates. A rule change of New York needs a
  `cargo update -p jiff-tzdb` and a new build.
- A `jiff` date or time that reads the system zone, such as `Zoned::now()`, falls back to UTC,
  because `tz-system` is off. The code must name the zone with `tz::get!`.
- Until the last slice merges, the code converts between `chrono` and `jiff` values at each place
  that has moved. Each slice removes the conversions of its crate.

## Open Questions

- `chrono` writes the fraction of a second with 3, 6, or 9 digits. `jiff` writes the fewest digits.
  The `storage` slice ([STA-593](https://linear.app/state-machine/issue/STA-593)) checks whether a
  stored timestamp has a fraction, and decision 4 holds it to the same string.

## Sources

- `cargo audit` on `main` at `850a823`, 2026-11-24: RUSTSEC-2026-0543 (`chrono`) and
  RUSTSEC-2026-0555 (`chrono-tz`).
- `cargo info jiff`, 2026-11-24: version 1.0.3 and its default features.
- `jiff-tzdb` 1.0.0, `tzname.rs`: `VERSION = "2026f"`.
- The builds and sizes in "Options Considered", 2026-11-24.
- [ADR 0002](0002-static-binaries.md), "Consequences": a binary compiles in the data it needs.
