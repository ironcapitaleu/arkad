---
last-verified: 2026-08-30
update-frequency: on-code-change
---

# CPU Profiling

This reference answers one question: where does CPU time go? The tool is `samply`, a sampling
profiler.

## Tool: samply

`samply` wraps the sampling facility of the platform. That is `perf_event` on Linux and the native
sampling APIs on macOS. Its interface is much easier to read than raw `perf`. It runs on x86-64 Linux
and on Apple Silicon, and the team uses both.

```sh
cargo install --locked samply
```

### The `profiling` Cargo Profile

The workspace `[profile.release]` sets `strip = true`, so a profile of a release binary shows hex
addresses and nothing else. For this reason the **workspace root** `Cargo.toml` carries
`[profile.profiling]`, which keeps release codegen and retains the debug symbols. It is committed, so
you add nothing before your first run.

Do not profile a `dev` build instead. A `dev` build measures unoptimised code that we never ship.

### Running It

```sh
cargo build --profile profiling --features tracing-logging --bin stream_etl
samply record ./target/profiling/stream_etl
```

**This is a live run.** `stream_etl` has no fixture mode and no offline flag. It drives all 469 CIKs
against EDGAR, so the run takes at least 52 seconds at the 110 ms pacing gate. This is the
exploratory exception that the invariant in `SKILL.md` allows. Profile with this command. Never
assert a number that came from it.

`--features tracing-logging` is required. Both `stream_etl` and `stream_extract` declare
`required-features = ["tracing-logging"]` in `sec/Cargo.toml`, and cargo refuses to build either
binary without the flag. The `extraction` binary has no such requirement.

`samply record` opens a local Firefox Profiler window when the process exits. To capture without that
window, run `samply record --save-only -o profile.json <cmd>`. Then run `samply load profile.json`
to view the result.

### Traps

- **`strip = true` in release.** This setting is the reason the `profiling` profile exists. If every
  frame is a hex address, this setting is the cause.
- **Omitting `--features tracing-logging`.** Cargo fails with
  `target requires the features: 'tracing-logging'` and builds nothing. This applies to every `cargo`
  command that names `stream_etl` or `stream_extract`, including `--release` builds.
- **`--save-only` writes an unsymbolized profile.** samply resolves symbols in its own interface, so
  a saved profile has an empty `nativeSymbols` and hex frames. This is fine when you view the file
  with `samply load`. It is awkward in a script, where you need `samply load` or `addr2line` by hand.
- **`perf_event_paranoid` on Linux.** samply detects a restrictive setting and prints the exact fix
  instead of an errno. Follow its instructions, which usually means lowering
  `kernel.perf_event_paranoid`.
- **Profile the right thing.** `[profile.profiling]` inherits `panic = "abort"` from release. This is
  harmless for samply and fatal for any tool that writes its report on `Drop`.

## Check Your Measurement Setup Before You Trust a Profile

The code you write to drive a measurement can feed the application something that production never
feeds it. The profile then describes that driver instead of the application, and it still looks
plausible. Read the top frames and ask whether they fit the workload, before you draw any conclusion.
Frames from a phase that the workload never runs are the clearest sign.
