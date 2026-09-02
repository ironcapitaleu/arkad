---
last-verified: 2026-08-30
update-frequency: on-code-change
---

# Memory profiling

Three separate questions are easy to confuse. Each one has a different answer.

1. **How much memory at peak?** Use `/usr/bin/time -v`, on Linux only. It is free, it needs no
   install, and it works today. It works only against a live run. See the blocker below. On macOS,
   `/usr/bin/time` is the BSD build and has no `-v` flag. Run `/usr/bin/time -l` there and read
   `maximum resident set size`. The BSD build reports that value in bytes, not in kilobytes.
2. **Where do allocations come from?** The project adopted no tool. This answer is not available.
3. **How large is this type, and does it live on the stack or the heap?** Use `size_of` and
   `align_of`. No tooling exists for this question.

**Status: partly covered.** Only questions 1 and 3 have an answer today. Do not promise allocation
attribution before the team agrees to adopt a tool.

## Peak resident memory, against a live run

**Peak resident memory** is the largest amount of physical RAM that the process held at any one
instant during the run. Many tools shorten the term to *peak RSS*, for "resident set size". It is not
the total that the run allocated. It is not the memory still held at exit.

```sh
cargo build --release --features tracing-logging --bin stream_etl
/usr/bin/time -v ./target/release/stream_etl
```

**This is a live run.** `stream_etl` has no fixture mode, and it drives all 469 CIKs against EDGAR.
The run takes about 52 seconds, because the rate limiter paces those requests 110 ms apart. That
duration is how long you wait. It is not a number this command measures. The run is the exploratory
exception that the invariant in `SKILL.md` allows. Read the memory number as an order of magnitude.
Never assert against it as a baseline.

`--features tracing-logging` is required. `stream_etl` declares `required-features` for it, and cargo
refuses to build the binary without the flag. See `references/profiling/cpu.md`.

Read the `Maximum resident set size (kbytes)` line. The measurement costs 30 seconds and no install.
Run it first for any question of the form "how much memory does X use". It often answers the question
outright, and a heap profiler then only confirms the result.

This command measures the whole process, so it cannot attribute memory to one component. Read it
next to the axis and component table in `SKILL.md`. Peak resident memory is a binary-level number.

## Struct layout

No profiler answers the question "does this type belong on the stack or the heap". Use
`std::mem::size_of::<T>()` and `align_of::<T>()`, then reason about the result. The answer affects
CPU as well as memory. A large type moved by value costs a memcpy. Boxing it trades that memcpy for
an allocation and a pointer chase. Measure the CPU effect with samply, described in
`references/profiling/cpu.md`. Do not measure it with a memory tool.

## Known blocker: a reproducible memory measurement

The command above runs today, but only by calling EDGAR. The number carries network
non-determinism, so you cannot assert against it. The same multi-CIK run without the live SEC API
needs a client-injection seam, and that seam does not exist. The state contexts hold the **concrete**
`SecClient` struct instead of a generic `C: SecClient`, so no fake can pass through them. The
existing fakes are placeholders as well, with `Request = ()` and `Response = String`. They must
become fixture-backed first. That refactor is its own ticket. Say so instead of improvising a
measurement setup of your own.
