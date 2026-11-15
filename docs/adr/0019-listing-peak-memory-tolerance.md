# ADR 0019: The Peak Memory Check of a Listing Allows 50% With Four Times the Filings

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([ironcapitaleu/arkad#577](https://github.com/ironcapitaleu/arkad/pull/577)) |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | None. [ironcapitaleu/arkad#577](https://github.com/ironcapitaleu/arkad/pull/577) acts on it |

## Context

`repository_listing.md`, "Memory", says that the peak memory of a listing grows with the widest
directory, not with the count of files. The integration test `storage/tests/listing_peak_memory.rs`
checks it: it lists a store and a copy with twice the filings for the same companies, and the two
peaks must be within 10%.

On 2026-11-15 the test failed once in CI on PR #572, at `5bc36cc`, with peaks of 28,147 and 33,751
bytes. The same job passed on `9f82554` and on a re-run of `5bc36cc`. That commit changes no code
that the test compiles.

A listing keeps up to 16 metadata reads in flight (`METADATA_READS_IN_FLIGHT`). Each read runs on
the blocking thread pool of tokio. How many reads hold their buffers and their task at the peak
depends on how the threads run. On a busy machine, more reads finish before the caller takes their
items. The window has a fixed size, so this part of the peak does not grow with the store. But it
is larger than 10% of the peak.

The choice binds the test and the "Memory" rule of `repository_listing.md`. The spread rests on a
measurement of one tokio version, so it can go out of date.

## Decision

1. **The check lists a store and a copy with four times the filings for the same companies.**
2. **The two peaks must be within 50% of each other.** The tolerance covers the spread of the read
   window, and a listing that keeps one allocation of 16 bytes per key still fails.

## Options Considered

Measured on 2026-11-15 in a cloud container with 4 CPUs, tokio 1.54.1, on `5bc36cc`. "Busy" means
4 busy loops in `sh` next to the test, one per CPU.

| Option | Peaks of one store, busy | Runs failed, busy | Smallest kept bytes per key that fail | Time of one run, busy |
| ------ | ------------------------ | ----------------- | ------------------------------------- | --------------------- |
| **A (chosen):** 4× the filings, 50% | 28,095 to 37,782 bytes | 0 of 8 | 10 | 14 to 18 s |
| B: 2× the filings, 10% (the old check) | 28,095 to 37,782 bytes | 1 of 8 | 4 | 12 to 14 s |
| C: one blocking thread, 2×, 10% | 33,501 to 37,152 bytes | Ratio 0.918 to 1.101 in 40 rounds | 4 | Not measured |
| D: a pause of 200 µs before each item, 2×, 10% | Ratio 0.878 to 1.183 | Not run | 4 | 40 s for 20 rounds, idle |
| E: `#[ignore]` the test | None | None | None. CI does not run it | 0 s |

"Smallest kept bytes per key that fail" assumes the worst pairing of measured peaks: the first store
at 37,782 bytes and the second store with no spread. The second store of A holds 2,700 more keys
than the first, and the second store of B holds 900 more.

- A keeps the check in CI, and its tolerance is above the measured spread with room to spare. Every
  heap allocation is at least 8 bytes, and a kept `String` or `Box` per key is at least 16.
- B fails on a busy runner with no change to the listing.
- C and D keep the spread. A blocking thread or a pause changes when reads finish, not how many
  reads can hold their buffers at once.
- E stops the check. The `testing` skill keeps a test of under 5 seconds on local files in CI.

## Consequences

- The check no longer fails on a busy runner with no change to the listing.
- A listing that keeps less than about 10 bytes per key can pass. No allocation is that small,
  but a listing that grows a shared buffer by a few bytes per key can pass.
- The test writes 1,200 filing directories and 7,200 files for the second store, and takes a few seconds
  longer.

## Open Questions

- A new tokio version, a new value of `METADATA_READS_IN_FLIGHT`, or a larger metadata file changes
  the spread. If the test fails on a busy runner again, measure the spread again and amend this
  ADR.

## Sources

- CI job `integration-tests (ubuntu-22.04)`, run 43711937919, on `5bc36cc`, read on 2026-11-15.
- `storage/tests/listing_peak_memory.rs` and
  `storage/src/implementations/repository/filesystem/implementations/list_repository/mod.rs` at
  `5bc36cc`.
- dhat heap profiles of 40 listings of the 2× store, busy, 2026-11-15. At the highest peak, reads
  of the listing held 2,968 more bytes in tokio read tasks than at the lowest peak, and 2,520 more
  in `read_entry` futures.
- `cargo test -p storage --test listing_peak_memory`, 8 runs busy for A and for B, 2026-11-15.
