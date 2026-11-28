# ADR 0051: The Offset `+00:00` as the Arrow Time Zone of Silver

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project owner confirms |
| Date | 2026-11-27 |
| Deciders | The project owner |
| Ticket | [STA-553](https://linear.app/state-machine/issue/STA-553), accessed 2026-11-27 |

## Context

Silver stores every timestamp as an Arrow `timestamp` with the unit milliseconds and a time zone.
The columns are `accepted` and `loaded_at` of `as_is_facts` and `filings`, `sic_retrieved_at` of
`filings`, and `computed_at` and `written_at` of the series datasets and `unit_manifests`. Each value
is a UTC instant.

The time zone of an Arrow type is a string. It is part of the schema, so it is part of each
Parquet file that the silver writer stores, and of the schema snapshot of each dataset.

`arrow` 60.2.0 reads two kinds of time zone strings:

- An offset, such as `+00:00`. `arrow` reads it with its default features.
- A name, such as `UTC`. `arrow` reads it only with its feature `chrono-tz`.

[ADR 0047](0047-jiff-for-dates-and-time-zones.md) removed `chrono-tz` from every crate. The crate
has the advisory RUSTSEC-2026-0555.

## Decision

1. **Every timestamp column of silver has the Arrow time zone `+00:00`.** The type is
   `timestamp[ms, +00:00]`. The constant `sec::datasets::UTC_TIME_ZONE` holds the string.
2. **`arrow` builds without its feature `chrono-tz`.**

## Options Considered

Measured on 2026-11-27 on `main` at `e398432`, with `arrow` 60.2.0 and its default features, in a
unit test that parses each string with `arrow::array::timezone::Tz::from_str`.

| Option | Time zone string | `arrow` without `chrono-tz` | New dependency |
| ------ | ---------------- | --------------------------- | -------------- |
| **A (chosen)** | `+00:00` | Reads it | None |
| B | `UTC` | Fails: "only offset based timezones supported without chrono-tz feature" | `chrono-tz`, with RUSTSEC-2026-0555 |
| C | None | Reads it | None |

- A against B: B brings back `chrono-tz`, which ADR 0047 removed for its advisory.
- A against C: a timestamp without a time zone is a local time in Arrow. A reader cannot tell that
  the value is UTC.

## Consequences

- The schema snapshot of each dataset prints `timestamp[ms, +00:00]`.
- A reader that compares the time zone string with `UTC` finds no match. It must compare the
  instant, or accept both strings.
- A move to `UTC` later changes the schema of each dataset, and needs a new schema version.

## Open Questions

- How Polars, DuckDB, and BigQuery show a column with the time zone `+00:00` is not measured. The
  silver writer measures it when it writes the first Parquet file.

## Sources

- `arrow-array` 60.2.0, `Cargo.toml`: `chrono-tz` is an optional dependency.
- Unit test on 2026-11-27, `main` at `e398432`: `Tz::from_str("UTC")` returned the error above,
  and `Tz::from_str("+00:00")` returned `Ok`.
- [ADR 0047](0047-jiff-for-dates-and-time-zones.md): `chrono-tz` and RUSTSEC-2026-0555.
