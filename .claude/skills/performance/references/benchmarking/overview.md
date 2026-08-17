---
source: STA-127 findings §1, §2, §7 (framework and CI survey) — surveyed, not decided; sec/tests/rate_limiter.rs
last-verified: 2026-08-17
update-frequency: on-decision
---

# Benchmarking — Placeholder

**Status: nothing exists.** No framework is adopted, no benchmarks are written, no baseline is stored,
and no benchmark runs in CI. This file is a placeholder that records what benchmarking is *for* here,
what has already been surveyed, and what must be decided before anything is added.

Do not add a benchmark harness, a CI bench job, or a threshold gate without settling the open decisions
below. When one is settled, move it out of the open list and into an "Adopted" section here.

## What benchmarking is for in this project

Profiling produces an artifact a human interprets. Benchmarking turns one number from that artifact
into an assertion that fails when it moves. The pipeline is therefore always:

**profile → find the number worth watching → assert it → only the assertion goes in CI.**

The purpose is regression detection, not discovery. A benchmark added before the corresponding profile
guards a number nobody has established is meaningful.

## Constraints any future design must respect

These are conclusions from measurement, not preferences — they rule out the obvious approaches.

- **Wall-clock on shared CI runners cannot resolve our code.** Run-to-run variance on shared GitHub
  runners is commonly ±10–50 %, and everything we own is sub-millisecond. A wall-clock benchmark of it
  measures the runner, and gating on one guarantees flaky red PRs.
- **The rate-limited end-to-end path must never be benchmarked for throughput.** It measures
  `MIN_REQUEST_INTERVAL` (a config constant) plus SEC network latency. A deliberate config change would
  show up as a "regression"; a real code regression would hide underneath a ~53 s floor.
- **The pacing constant is already guarded by a test, not a benchmark.** `sec/tests/rate_limiter.rs`
  asserts that N permits take at least (N − 1) × interval. It measures real wall-clock via
  `Instant::now()`, yet it does not contradict the bullet above, because every assertion is a **lower
  bound**: a leaky-bucket limiter can never release permits *faster* than its configured rate, so
  runner noise can only push elapsed time up, never under the bound. No benchmark machinery, and it
  already runs in the normal suite — the correct tool for a config invariant.
- **Fixtures, never the live SEC API.** Non-deterministic, rate-limited, and the rate limit is often the
  thing under test.
- **Allocation count is a good proxy for CPU cost on the parse path**, where ~86 % of CPU is
  allocation-driven. An allocation-count assertion in a plain `#[test]` is cheap, deterministic, and
  cross-platform — a candidate worth weighing against a real benchmark framework, since it needs no
  bench job, no hosted service, and no Valgrind.

## Open decisions

1. **Do we need a framework at all, or do assertions in ordinary tests suffice?** Answer this first —
   it may make 2–4 moot.
2. **Which framework**, if one is needed. Surveyed but not chosen: `criterion` (mature, wall-clock,
   baseline storage), `divan` (better ergonomics, weaker baselines), `iai-callgrind` (deterministic
   instruction counts, needs Valgrind, **does not run on Apple Silicon**).
3. **Where the baseline lives.** Surveyed: CodSpeed and Bencher (hosted history, PR comments,
   thresholding — a third-party service holding internal numbers), `critcmp` plus a `gh-pages`-style
   store (zero external dependency, we own the comparison and storage logic).
4. **What is measured.** Wall-clock, instruction counts, or allocation counts. Determines 2 and 3 more
   than any other choice.
5. **Trigger and threshold.** When it runs (path-filtered PR job, opt-in label, weekly cron) and what
   counts as a regression. Deterministic measurement tolerates a tight bound; wall-clock does not.

## Rollout phasing

The agreed sequence — do not skip ahead:

| Phase | What | Status |
| --- | --- | --- |
| 0 | Skill and reference material as the source of truth for performance knowledge | in progress |
| a | Profile real components along real axes, on demand | available for CPU |
| b | Make results visible to CI and to review — **non-blocking**, nothing enforced | not started |
| c | Enforced CI gates, against a chosen baseline | not started, may never happen |

Phase b means results are *visible*, never that a PR can fail on them. Any gate belongs to phase c and
requires an explicit decision, plus an override for intentional changes such as a deliberate
`MIN_REQUEST_INTERVAL` tweak, which is a config move rather than a regression.

## Prior survey

The full framework comparison, CI-integration options, threshold models, and the tiered-triggering
proposal live in the
[STA-127 findings document](https://linear.app/state-machine/document/findings-benchmark-pipeline-performance-for-extract-and-critical-ed0bbbfbfa04)
(§1, §2, §7). It is a survey, not a decision — read it when working through the open decisions above,
and record the outcome here rather than there.
