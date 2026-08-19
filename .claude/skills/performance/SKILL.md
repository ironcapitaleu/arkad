---
name: performance
description: >
  Use when the user asks to "profile", "benchmark", "measure performance", "find the bottleneck",
  "check for a performance regression", "how much memory does X use", "why is X slow", or wants to
  measure CPU, memory, or wall-time cost of a binary, state, state machine, function, or struct layout.
  Also the source of truth for what we currently know about performance in this project.
version: 0.1.0
argument-hint: "[profile|benchmark] [component]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# Performance: Profiling and Benchmarking

## Purpose

This skill is the project's source of truth for performance engineering: what we measure, along which
axes, with which tools, and what we have already learned. It exists so that a measurement is never
started from scratch and never produces a number nobody can interpret.

Two pillars, deliberately separate:

- **Profiling** — measuring where cost goes along an axis (CPU, memory, wall-time), for a component,
  with a stated goal. Output is an artifact a human reads. There is no pass/fail.
- **Benchmarking** — asserting a number profiling found, so a regression fails loudly. Output is a
  test result. **Not yet implemented in this project** — see `references/benchmarking/overview.md`.

Profiling finds the number worth watching. Benchmarking asserts it. Only the assertion can ever go in CI.

## Current Maturity — read this before promising anything

| Capability | Status |
| --- | --- |
| CPU profiling | ✅ Available — `samply`, validated on a real workload |
| Memory profiling | ⚠️ Partial — peak RSS only; no allocation-attribution tool adopted |
| Wall-time measurement | ⚠️ Available but rarely meaningful (see Invariants) |
| Storage / I/O profiling | ❌ Not covered — no measurements exist |
| Benchmarking | ❌ No framework, no benchmarks, no baseline store |
| CI integration | ❌ Nothing runs in CI |

Tooling is intentionally thin and **swappable**. The knowledge in this skill is the asset; the tool that
currently produces it is not. When a tool is replaced, the recorded numbers stay valid — only the
"how to run it" sections change.

## Context Gathering — always in this order

Never start a tool before all three are answered. A profile without a goal is an artifact nobody acts on.

1. **Goal** — why measure? Valid goals: find a bottleneck, establish a baseline, confirm or refute a
   suspicion, investigate a suspected regression. "Because it might be slow" is not a goal; ask for one.
2. **Axis** — CPU, memory, or wall-time. If the user says "profile X" without an axis, ask.
3. **Component** — binary, state machine / super-state, state, function/method, or struct layout.

If the user names the component but not the axis, propose one from the matrix below and say why.

## Axis × Component — which combinations are meaningful

Not every combination is worth measuring. `✅` do it, `⚠️` only with a caveat, `❌` don't — the number
would be misleading.

| Component | CPU | Memory | Wall-time |
| --- | --- | --- | --- |
| **Binary** (`stream_etl`, `stream_extract`) | ✅ samply on the whole run | ✅ peak RSS | ⚠️ dominated by the rate-limiter constant, not our code |
| **StateMachine / SuperState** | ✅ | ⚠️ needs a client-injection seam to run without network | ❌ gate-bound — measures config and the internet |
| **State** (single) | ✅ if pure (no I/O) | ⚠️ only via the enclosing run | ❌ sub-millisecond, below any noise floor |
| **Function / method** | ✅ the common case | ⚠️ same | ❌ sub-millisecond |
| **Struct layout** | ➖ indirect (stack vs heap affects CPU) | ✅ `size_of` / `align_of`, boxing decisions | ❌ |

The `❌` column is the single most important thing in this table. The Extract path is paced by a global
rate limiter at ~110 ms/permit; a wall-clock measurement of anything under it re-measures that constant
plus SEC network latency, not the code. Everything we own is sub-millisecond.

## Mode: Profiling

1. Confirm goal, axis, component (above).
2. Open the matching reference and follow it — it carries the tool invocation, the required cargo
   profile, the known traps, and the numbers we already have:
   - CPU → `references/profiling/cpu.md`
   - Memory → `references/profiling/memory.md`
3. **Sanity-check the rig before trusting the profile.** If the measurement code differs from what
   production does, the profile describes the rig. Check the top frames make sense for the workload
   before drawing any conclusion.
4. Report: the number, the machine it was measured on, the date, and what it means for the goal from
   step 1. A profile reported without its machine and date is not reusable.
5. If the finding is durable, record it in the reference file's "What we have measured" section.

## Mode: Benchmarking

Currently a placeholder. Read `references/benchmarking/overview.md` — it states what exists (nothing),
what was surveyed, and what must be decided before a framework is adopted. Do not add a benchmark
framework, a CI bench job, or a threshold gate without going through the open decisions listed there.

If the user asks to benchmark something today, say plainly that no framework is adopted yet, offer to
profile it instead, and offer to work through the open decisions in that file.

## Critical Invariants

- **Profiling never runs in CI on a pull request.** Profilers cost 20–50× in runtime and produce
  artifacts with no threshold to pass or fail. There is nothing for CI to gate on.
- **Never wall-clock a rate-limited path for throughput.** It measures `MIN_REQUEST_INTERVAL`, which is
  a config constant, not code. Changing it is a config decision, never a "regression".
- **Separate pacing floor from code overhead.** The pacing floor is intentional and config-driven; code
  overhead is everything else. Only code overhead is a regression candidate.
- **Never benchmark or profile against the live SEC API.** Non-deterministic, rate-limited, and the rate
  limit is often the thing under test. Use a checked-in fixture.
- **Symbols are required.** The `release` profile sets `strip = true`, which reduces every profile to
  hex addresses. Use the profiling cargo profile documented in `cpu.md`.
- **State the machine.** Numbers from Apple Silicon and x86-64 Linux are not comparable. Every recorded
  measurement carries its machine and date.

## Proactive Behavior

- When asked to "make X faster", **profile before changing anything.** Two prior predictions about
  where this pipeline's cost lives were both wrong; measurement corrected both.
- When a number in a reference file contradicts a fresh measurement, flag it and propose an update
  rather than silently trusting either.
- When someone proposes a CI performance gate, point at the Invariants — the gate must assert a
  deterministic number, not a wall-clock one.
- When a measurement reveals a *functional* problem rather than a performance one, say so and recommend
  a separate ticket. Performance work should not absorb correctness bugs.
- If asked to measure an axis or component marked `❌` above, say why the number would mislead and
  propose the meaningful alternative.

## Authoritative Sources

- [The Rust Performance Book — Profiling](https://nnethercote.github.io/perf-book/profiling.html) —
  a catalogue of what exists, not a ranking. Each question has one conventional answer.
- [The Rust Performance Book — Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)
- [samply](https://github.com/mstange/samply) — the adopted CPU profiler.
- [perf wiki](https://perfwiki.github.io/main/) — the substrate samply drives on Linux.
- [STA-127 findings document](https://linear.app/state-machine/document/findings-benchmark-pipeline-performance-for-extract-and-critical-ed0bbbfbfa04)
  — the SPIKE this skill was distilled from. Retains the full rate-limiter analysis and the tool survey.

## Self-Improvement

This skill is expected to be wrong in places and to be corrected through use. It is version-tracked so
that corrections accumulate instead of being re-derived.

After any profiling or benchmarking session, ask whether to update this skill when:

- A measurement contradicts a recorded number, or the recorded number is stale for the current code.
- A tool is adopted, replaced, or dropped — update the Current Maturity table and the relevant reference.
- A new trap is hit (a flag that silently ruins output, a profile that describes the rig, a build
  setting that strips what you needed). These are the highest-value additions.
- A new axis or component becomes measurable — add the row to the matrix.
- A combination marked `❌` turns out to be meaningful after all, or vice versa.
- A benchmarking decision from `references/benchmarking/overview.md` is settled — move it out of the
  open list and into the adopted section.

Apply updates only after the user approves them.
