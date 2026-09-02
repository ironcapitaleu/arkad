---
name: performance
description: >
  Use when the user asks to "profile", "benchmark", "measure performance", "find the bottleneck",
  "check for a performance regression", "how much memory does X use", "why is X slow", or wants to
  measure CPU, memory, or wall-time cost of a binary, state, state machine, function, or struct layout.
version: 0.2.0
argument-hint: "[profile|benchmark] [component]"
allowed-tools: [Read, Write, Edit, Bash, AskUserQuestion]
---

# Performance: Profiling and Benchmarking

## Purpose

This skill holds the project's rules for performance work. It states what is worth measuring, along
which axes, and with which tools. Nobody starts a measurement from scratch, and no measurement
produces a number that nobody can read. This skill stores no results. See "Reporting".

It covers two separate activities:

- **Profiling** measures where cost goes along one axis, for one component, with a stated goal. The
  axes are CPU, memory, and wall-time. The output is a file a human reads. There is no pass or fail.
- **Benchmarking** asserts a number that profiling found, so a regression fails loudly. The output is
  a test result. This project has no benchmarking yet. See `references/benchmarking/overview.md`.

Profiling finds the number worth watching. Benchmarking asserts it. Only the assertion can go in CI.

## Current Maturity

Read this table before you promise anything.

| Capability | Status |
| --- | --- |
| CPU profiling | Available. Uses `samply`, validated on a real workload |
| Memory profiling | Partial. Peak resident memory only. No allocation-attribution tool adopted |
| Wall-time measurement | Available, but rarely meaningful. See "Critical Invariants" |
| Storage and I/O profiling | Not covered. No measurements exist |
| Benchmarking | None. No framework, no benchmarks, no baseline store |
| CI integration | None. Nothing runs in CI |

The tooling is thin on purpose, and any tool here can be swapped. The method in this skill is the
asset. The tool that implements it today is not.

## Context Gathering

Answer all three questions below before you start a tool. A profile without a goal is a file that
nobody acts on.

1. **Goal.** Why measure? These four goals are valid:
   - Find a bottleneck.
   - Establish a baseline.
   - Confirm or refute a suspicion.
   - Investigate a suspected regression.

   "Because it might be slow" is not a goal. Ask for a real one.
2. **Axis.** CPU, memory, or wall-time. If the user says "profile X" and names no axis, ask.
3. **Component.** A binary, a state machine, a super-state, a state, a function, or a struct layout.

If the user names the component but not the axis, propose one from the Axis × Component table. Say why.

## Axis × Component: Which Combinations Are Meaningful

Not every combination is worth measuring. **Yes** means measure it. **Caution** means measure it
only with the stated limit. **No** means the number will mislead you.

| Component | CPU | Memory | Wall-time |
| --- | --- | --- | --- |
| **Binary** (`stream_etl`, `stream_extract`) | Yes. samply on the whole run | Yes. Peak resident memory | Caution. The rate-limiter constant dominates, not our code |
| **StateMachine or SuperState** | Yes | Caution. Needs a client-injection seam to run without network | No. Gate-bound, so it measures config and the internet |
| **State** (single) | Yes, if the state is pure and does no I/O | Caution. Only through the enclosing run | No. Sub-millisecond, below any noise floor |
| **Function or method** | Yes. This is the common case | Caution. Same limit as a state | No. Sub-millisecond |
| **Struct layout** | Indirect. Stack against heap affects CPU | Yes. Use `size_of` and `align_of` for boxing decisions | No |

The wall-time column matters most. A global rate limiter paces the Extract path at about 110 ms per
permit. A wall-clock measurement below that gate re-measures the constant and the SEC network
latency, not our code. Everything we own runs in under a millisecond.

## Mode: Profiling

1. Confirm the goal, the axis, and the component.
2. Open the matching reference and follow it. Each one carries the tool command, the required cargo
   profile, and the known traps.
   - CPU: `references/profiling/cpu.md`
   - Memory: `references/profiling/memory.md`
3. Check your measurement setup before you trust the profile. If the code you wrote to drive the
   measurement differs from production, the profile describes that driver instead of the
   application. Confirm that the top frames make sense for the workload first.
4. Report two things. First the measurement: the number, the machine it ran on, and the date. Second
   what it means for the goal from step 1. A profile without its machine and date is not reusable. A
   profile without an interpretation leaves the reader to guess.

## Reporting

Results do not live in this skill. Numbers go stale, and the next reader takes a stale number in a
skill file as current fact. Report the measurement to the human. If the finding is durable, put it
in a findings document that humans read, not here.

These files hold the tool commands, the traps, and the list of measurements that are meaningful.
Those parts stay true between runs.

## Mode: Benchmarking

This mode is a placeholder. Read `references/benchmarking/overview.md`. It records what exists,
which is nothing, what the team surveyed, and what the team must decide before it adopts a
framework. Do not add a benchmark framework, a CI bench job, or a threshold gate before you work
through the open decisions in that file.

If the user asks to benchmark something today, say that the project adopted no framework yet. Offer
to profile the component instead. Offer to work through the open decisions in that file.

## Critical Invariants

- **Profiling never runs in CI on a pull request.** A profiler costs 20 to 50 times the runtime. It
  produces a file with no threshold to pass or fail, so CI has nothing to gate on.
- **Never wall-clock a rate-limited path for throughput.** The measurement returns
  `MIN_REQUEST_INTERVAL`, which is a config constant rather than code. Changing that constant is a
  config decision, never a regression.
- **Separate the pacing floor from code overhead.** The pacing floor is deliberate and config-driven.
  Code overhead is everything else. Only code overhead can be a regression.
- **Never make the live SEC API the subject of an asserted number.** It is non-deterministic and
  rate-limited. The rate limit is often the thing under test. Any number that you benchmark, gate,
  or record as a baseline must come from a checked-in fixture.

  Exploratory profiling of a real run is the one permitted exception. No fixture-driven target exists
  yet, so the commands in `references/profiling/` do call EDGAR. Say so when you use them, and state
  the cost. Every runnable binary drives all 469 CIKs listed in
  `sec/src/bin/stream_etl/pipeline/constants.rs`, which takes at least 52 seconds at the 110 ms
  pacing gate. Never promote a number measured that way into an assertion.
- **Debug symbols are required.** The `release` profile sets `strip = true`, which discards them.
  Every profile then shows hex addresses, because the profiler has no names to attribute samples to.
  Use the profiling cargo profile documented in `cpu.md`.
- **State the machine.** Numbers from Apple Silicon and from x86-64 Linux are not comparable. Every
  recorded measurement carries its machine and its date.

## Proactive Behavior

- When the user asks you to make something faster, profile it before you change anything.
  Predictions about where a pipeline spends its time are often wrong, including confident ones.
- When someone proposes a CI performance gate, point at the invariants. The gate must assert a
  deterministic number, not a wall-clock one.
- When a measurement reveals a functional bug rather than a performance problem, say so. Recommend a
  separate ticket. Performance work must not absorb correctness bugs.
- When asked to measure a combination marked **No** above, say why the number will mislead. Propose
  the meaningful alternative.

## Self-Improvement

Parts of this skill will be wrong, and use will correct them. The version field tracks those
corrections so that nobody derives them twice.

After a profiling or benchmarking session, ask whether to update this skill in these cases:

- The team adopts, replaces, or drops a tool. Update the maturity table and the matching reference.
- You hit a new trap. These additions are worth the most. Three examples:
  - A flag that ruins the output silently.
  - A profile that describes the measurement setup instead of the application.
  - A build setting that strips what you needed.
- A new axis or component becomes measurable. Add the row to the table.
- A combination marked **No** turns out to be meaningful, or a combination marked **Yes** turns out
  to mislead.
- The team settles a benchmarking decision from `references/benchmarking/overview.md`. Move it out
  of the open list and into the adopted section.

Apply an update only after the user approves it.
