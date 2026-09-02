---
last-verified: 2026-08-30
update-frequency: on-code-change
---

# CPU Profiling

Answers: **where does CPU time go?** Tool: `samply`, a sampling profiler.

## Tool: samply

`samply` is a wrapper around the platform's sampling facility (`perf_event` on Linux, the native
sampling APIs on macOS) with a far better UI than raw `perf`. It works on both x86-64 Linux and Apple
Silicon, which matters because the team spans both.

```sh
cargo install --locked samply
```

### The `profiling` cargo profile

The workspace `[profile.release]` sets `strip = true`. Profiling a release binary therefore yields a
flamegraph of hex addresses and nothing else. The **workspace root** `Cargo.toml` carries
`[profile.profiling]` for this reason — release codegen with debug symbols retained. It is committed, so
nothing needs adding before a first run.

Profiling a `dev` build instead is not a substitute — it measures unoptimised code that we never ship.

### Running it

```sh
cargo build --profile profiling --features tracing-logging --bin stream_etl
samply record ./target/profiling/stream_etl
```

**This is a live run.** `stream_etl` has no fixture mode and no offline flag — it drives all 469 CIKs
against EDGAR, so expect a ~52 s floor at the 110 ms pacing gate. That is the exploratory exception
allowed by the invariant in `SKILL.md`: profile with it, never assert a number from it.

`--features tracing-logging` is required, not optional: both `stream_etl` and `stream_extract` declare
`required-features = ["tracing-logging"]` in `sec/Cargo.toml`, and cargo refuses to build them without
it. The `extraction` binary has no such requirement.

`samply record` opens a local Firefox-Profiler UI when the process exits. To capture without the UI,
`samply record --save-only -o profile.json <cmd>`, then `samply load profile.json` to view it.

### Traps

- **`strip = true` in release** — the reason the `profiling` profile exists. If every frame is a hex
  address, this is why.
- **Omitting `--features tracing-logging`** — cargo fails with
  `target requires the features: 'tracing-logging'` rather than building anything. Applies to every
  `cargo` command that names `stream_etl` or `stream_extract`, `--release` builds included.
- **`--save-only` writes an *unsymbolized* profile.** Symbolication happens in samply's UI, so a saved
  profile has an empty `nativeSymbols` and hex frames. Fine interactively (`samply load`), awkward in
  scripts — headless use needs `samply load` or manual `addr2line`.
- **`perf_event_paranoid` on Linux.** samply detects a restrictive setting and prints the exact fix
  rather than an errno; follow its instructions (typically lowering `kernel.perf_event_paranoid`).
- **Profile the right thing.** `[profile.profiling]` inherits `panic = "abort"` from release. Harmless
  for samply, fatal for any tool that reports on `Drop`.

## Check your measurement setup before trusting a profile

If the code you wrote to drive the measurement feeds the application something production never feeds
it, the profile describes that driver rather than the application — and it looks entirely plausible
while doing so. Read the top frames and ask whether they make sense for the workload *before* drawing
any conclusion. Frames from a phase the workload does not perform are the clearest tell.
