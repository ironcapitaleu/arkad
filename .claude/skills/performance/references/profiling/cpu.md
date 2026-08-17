---
source: STA-127 findings §8 (hands-on tool evaluation, 2026-08-07); workspace Cargo.toml; sec/Cargo.toml
last-verified: 2026-08-17
update-frequency: on-measurement, on-tool-change
---

# CPU Profiling

Answers: **where does CPU time go?** Tool: `samply`, a sampling profiler. Adopted after a hands-on
evaluation that filtered ten candidates down to one for this axis.

## Tool: samply

`samply` is a wrapper around the platform's sampling facility (`perf_event` on Linux, the native
sampling APIs on macOS) with a far better UI than raw `perf`. It works on both x86-64 Linux and Apple
Silicon, which matters because the team spans both.

```sh
cargo install --locked samply
```

### The `profiling` cargo profile

The workspace `[profile.release]` sets `strip = true`. Profiling a release binary therefore yields a
flamegraph of hex addresses and nothing else. The **workspace root** `Cargo.toml` carries a dedicated
profile for this reason — it is committed, so nothing needs adding before a first run:

```toml
[profile.profiling]
inherits = "release"
debug = true
strip = false
```

Same optimisation level as release, but symbols survive. Profiling a `dev` build instead is not a
substitute — it measures unoptimised code that we never ship.

### Running it

```sh
cargo build --profile profiling --features tracing-logging --bin stream_etl
samply record ./target/profiling/stream_etl
```

`--features tracing-logging` is required, not optional: both `stream_etl` and `stream_extract` declare
`required-features = ["tracing-logging"]` in `sec/Cargo.toml`, and cargo refuses to build them without
it. The `extraction` binary has no such requirement.

`samply record` opens a local Firefox-Profiler UI when the process exits. To capture without the UI,
`samply record --save-only -o profile.json <cmd>`, then `samply load profile.json` to view it.

### Traps

- **`strip = true` in release** — the reason the `profiling` profile exists. If every frame is a hex
  address, this is why.
- **Omitting `--features tracing-logging`** — cargo fails with `requires the features:
  'tracing-logging'` rather than building anything. Applies to every `cargo` command that names
  `stream_etl` or `stream_extract`, `--release` builds included.
- **`--save-only` writes an *unsymbolized* profile.** Symbolication happens in samply's UI, so a saved
  profile has an empty `nativeSymbols` and hex frames. Fine interactively (`samply load`), awkward in
  scripts — headless use needs `samply load` or manual `addr2line`.
- **`perf_event_paranoid` on Linux.** samply detects a restrictive setting and prints the exact fix
  rather than an errno; follow its instructions (typically lowering `kernel.perf_event_paranoid`).
- **Profile the right thing.** `[profile.profiling]` inherits `panic = "abort"` from release. Harmless
  for samply, fatal for any tool that reports on `Drop`.

## What we have measured

**Machine: x86-64 Arch Linux workstation. Date: 2026-08-07.** Not comparable to Apple Silicon numbers.
Workload: the CPU-bound parse path fed from the checked-in 5.3 MB Berkshire fixture
(`sec/src/lib/tests/fixtures/data/raw_input/CIK0001067983.json`) — no network, no rate limiter.

Wall-clock, 30 iterations, per filing:

| Phase | Cost |
| --- | --- |
| Deserialize (`serde_json::from_str` of the response body) | **14.8 ms** |
| Parse (`ParseCompanyFacts` concept resolution) | **1.18 ms** |
| Serialize (`to_value` of the state, per event) | **2.8 µs** |

samply, 703 samples:

| Frame group | Share |
| --- | --- |
| `libc` (malloc / free) | 43.7 % |
| serde_json deserializer | 33.3 % |
| BTreeMap nodes / `RawVec` | 9.1 % |
| everything else | 13.4 % |

The single largest frame is `malloc`, called from allocating a `String` for **every key in every JSON
object** (10.0 % on its own).

### What this means

- **~86 % of CPU is building a `serde_json::Value` and the allocation traffic that entails.** None of
  our own logic appears until far below that. Optimising code we wrote would be optimising the 14 %.
- **Deserialization is over 12× the parse cost**, in the stage previously assumed to be the cheapest.
  Two earlier predictions about where the cost lives were wrong; both were corrected by measurement.
- **None of this is a bottleneck today.** Roughly 8 s of CPU across a 469-CIK batch, spread over cores,
  under a ~53 s rate-limiter floor. Recorded as a baseline to notice future drift, not as a problem.
- **Known optimisation direction, deliberately not taken:** if deserialization ever mattered, the lever
  is not building a full `serde_json::Value` — deserialize into typed structs or a borrowed
  representation. Not worth doing while the path is gate-bound.

## Sanity-check the rig before trusting a profile

The first symbolized profile of this workload showed ~13 % of samples in `serde_json::ser::*` — which
is impossible for a deserialize-only workload. Cause: the measurement harness fed
`response.to_string()` where production feeds the raw body text. **The profiler found a bug in the
measurement rig before it found anything about the application.**

Read the top frames and ask whether they make sense for the workload *before* drawing a conclusion.

## Escalation — tools we did not adopt

Deliberately not installed. Each has a trigger that would earn it back in:

| Tool | Would answer | Adopt when |
| --- | --- | --- |
| `cachegrind` | cache misses, branch mispredicts | a profile shows a tight numeric loop dominating — ours is JSON parsing and map inserts, where it has no reachable action |
| `cargo-flamegraph` | same question as samply | never; it is another `perf` wrapper with a worse UI |
| `tokio-console` | is the async runtime stalling? | a fan-out workload shows unexplained latency. The current question was already settled by measurement: thundering herd cost zero, and 16 CPU hogs oversubscribing all cores cost zero at the rate-limiter gate |
| `perf` (raw) | samply's substrate | only if samply itself is in the way |
