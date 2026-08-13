---
source: STA-127 findings §8 (2026-08-07); §6 live validation run (2026-07-23)
last-verified: 2026-08-11
update-frequency: on-measurement, on-tool-change
---

# Memory Profiling

Answers three separate questions that are easy to conflate:

1. **How much memory at peak?** → `/usr/bin/time -v`. Free, no install, works today.
2. **Where do allocations come from?** → no tool adopted. See Escalation.
3. **How large is this type, and is it on the stack or the heap?** → `size_of` / `align_of`, no tooling.

**Status: partially covered.** Only (1) and (3) are available today. Do not promise allocation
attribution without first agreeing to adopt a tool.

## Peak resident memory — available today

```sh
cargo build --release --bin stream_etl
/usr/bin/time -v ./target/release/stream_etl
```

Read `Maximum resident set size (kbytes)`. Costs 30 seconds and no install. This is the correct first
measurement for any "how much memory does X use?" question — it often answers it outright, in which
case a heap profiler would only be confirmation.

Note this measures the whole process, so it cannot attribute memory to a component. Combine with the
axis × component matrix in `SKILL.md`: peak RSS is a binary-level number.

## Struct layout

For the "does this type belong on the stack or the heap?" question there is no profiler — use
`std::mem::size_of::<T>()` / `align_of::<T>()` and reason about it. Relevant to CPU as well as memory:
large types moved by value cost memcpy, and boxing trades that for an allocation and a pointer chase.
Measure the CPU consequence with samply (`references/profiling/cpu.md`), not with a memory tool.

## What we have measured

**Machine: x86-64 Arch Linux workstation. Date: 2026-08-07.** Workload: the CPU-bound parse path fed
from the 5.3 MB Berkshire fixture, one filing.

| Metric | Value |
| --- | --- |
| Total allocated | ~26 MB per filing |
| Allocation count | **~275,000 per filing** |
| Peak live heap | **24.4 MB** — ~4.8× the source document's own size |
| Bytes still live at exit | 1,064 → no leak |

> ⚠️ **These numbers came from a one-off `dhat-rs` run during the SPIKE. `dhat-rs` is not adopted and
> is not set up in the repo.** Recorded because the numbers are useful and were validated against
> samply's independent view; treat them as a historical observation, not as something reproducible with
> the current checkout.

### What this means

Allocation traffic is the dominant cost on this path — it corroborates samply's finding that ~86 % of
CPU is allocation-driven, from two independent tools. It also makes **allocation count a good proxy for
CPU cost here**, which is worth remembering if a regression assertion is ever built
(`references/benchmarking/overview.md`).

## Open questions

- **Peak RSS of a real 469-CIK run.** Free via `/usr/bin/time -v`, but needs a live SEC run. Nobody has
  run it.
- **Memory under concurrency.** The 24.4 MB figure is for a *single* filing. What 469 concurrent CIKs
  peak at is unknown, and measuring it without hitting the live SEC API needs a client-injection seam:
  the state contexts hold the **concrete** `SecClient` struct rather than a generic `C: SecClient`, so
  no fake can be threaded through. The existing fakes are also placeholders (`Request = ()`,
  `Response = String`) and would need to become fixture-backed. That refactor is its own ticket.

## Escalation — tools we did not adopt

| Tool | Would answer | Cost | Adopt when |
| --- | --- | --- | --- |
| `dhat-rs` | which call sites allocate, how many times | dev-dependency + two lines; **~42× slowdown** on this allocation-heavy workload; no Valgrind, cross-platform | peak RSS shows a problem and we need to attribute it. Trap: the report is written on `Drop`, and a profiling profile inheriting `panic = "abort"` yields *no report at all* |
| `heaptrack` | memory **over time**, not just aggregate | system package, Linux only | dhat's aggregate view cannot answer a question about growth or a spike |
| `massif` | peak and shape | Valgrind, 20–50× | probably never — overlaps the two above, and peak is free via `/usr/bin/time -v` |
| `bytehound` | overlaps heaptrack | — | no; less maintained |

Adopt the cheapest tool that answers the actual question, and record the decision here.
