---
last-verified: 2026-08-30
update-frequency: on-code-change
---

# Memory Profiling

Answers three separate questions that are easy to conflate:

1. **How much memory at peak?** → `/usr/bin/time -v` on **Linux only**. Free, no install, works
   today — but only against a live run (see the blocker below). On macOS `/usr/bin/time` is the BSD
   build and has no `-v`; use `/usr/bin/time -l` there and read `maximum resident set size`, which it
   reports in bytes rather than kilobytes.
2. **Where do allocations come from?** → no tool adopted. Not available.
3. **How large is this type, and is it on the stack or the heap?** → `size_of` / `align_of`, no tooling.

**Status: partially covered.** Only (1) and (3) are available today. Do not promise allocation
attribution without first agreeing to adopt a tool.

## Peak resident memory — available today, against a live run

**Peak resident memory** — often shortened to *peak RSS*, for "resident set size" — is the largest
amount of physical RAM the process ever had in use at one instant during the run. It is not the total
allocated over the run, and it is not the memory still held at exit.

```sh
cargo build --release --features tracing-logging --bin stream_etl
/usr/bin/time -v ./target/release/stream_etl
```

**This is a live run.** `stream_etl` has no fixture mode: it drives all 469 CIKs against EDGAR. Expect
it to *take* about 52 seconds, because the rate limiter paces those requests 110 ms apart — that
duration is how long you wait for the run, not something being measured here. It is the exploratory
exception allowed by the invariant in `SKILL.md`: the memory number is usable as an order of
magnitude, not as a baseline to assert against.

`--features tracing-logging` is required — `stream_etl` declares `required-features` for it and cargo
refuses to build the binary otherwise (see `references/profiling/cpu.md`).

Read `Maximum resident set size (kbytes)`. Costs 30 seconds and no install. This is the correct first
measurement for any "how much memory does X use?" question — it often answers it outright, in which
case a heap profiler would only be confirmation.

Note this measures the whole process, so it cannot attribute memory to a component. Combine with the
axis × component matrix in `SKILL.md`: peak resident memory is a binary-level number.

## Struct layout

For the "does this type belong on the stack or the heap?" question there is no profiler — use
`std::mem::size_of::<T>()` / `align_of::<T>()` and reason about it. Relevant to CPU as well as memory:
large types moved by value cost memcpy, and boxing trades that for an allocation and a pointer chase.
Measure the CPU consequence with samply (`references/profiling/cpu.md`), not with a memory tool.

## Known blocker: a *reproducible* memory measurement

The command above runs today, but only by going out to EDGAR, so the number carries network
non-determinism and cannot be asserted against. Measuring the same multi-CIK run without hitting the
live SEC API needs a client-injection seam that does not exist: the state contexts hold the
**concrete** `SecClient` struct rather than a generic `C: SecClient`, so no fake can be threaded
through. The existing fakes are also placeholders
(`Request = ()`, `Response = String`) and would need to become fixture-backed. That refactor is its own
ticket — say so rather than improvising a measurement setup of your own.
