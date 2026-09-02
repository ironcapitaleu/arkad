---
last-verified: 2026-08-30
update-frequency: on-decision
---

# Benchmarking: placeholder

**Status: nothing exists.** The project adopted no framework, wrote no benchmarks, stored no
baseline, and runs no benchmark in CI. This file records three things instead. What benchmarking is
for in this project, what the team already surveyed, and what the team must decide before it adds
anything.

Do not add a benchmark harness, a CI bench job, or a threshold gate before you settle the open
decisions below. When the team settles one, move it out of the open list into an "Adopted" section
in this file.

## What benchmarking is for in this project

Profiling produces a file that a human reads. Benchmarking turns one number from that file into an
assertion, and the assertion fails when the number moves. The order is always the same:

**profile, find the number worth watching, assert it, and put only the assertion in CI.**

The purpose is regression detection, not discovery. A benchmark written before its profile guards a
number that nobody showed to be meaningful.

## Constraints that any future design must respect

These are conclusions from measurement, not preferences. Each one rules out an obvious approach.

- **Wall-clock on a shared CI runner cannot resolve our code.** Run-to-run variance on a shared
  GitHub runner is commonly 10 to 50 percent. Everything we own runs in under a millisecond. A
  wall-clock benchmark measures the runner. A gate on that number produces flaky red pull requests.
- **Never benchmark the rate-limited end-to-end path for throughput.** That path measures
  `MIN_REQUEST_INTERVAL`, a config constant, plus the SEC network latency. A deliberate config change
  then looks like a regression. A real code regression hides under the pacing floor.
- **A test already guards the pacing constant, and no benchmark is needed.**
  `sec/tests/rate_limiter.rs` asserts that N permits take at least (N − 1) times the interval. It
  reads real wall-clock through `Instant::now()`, which does not contradict the bullet above. Every
  assertion in that file is a **lower bound**. A leaky-bucket limiter can never release permits
  faster than its configured rate. Runner noise can only push the elapsed time up, never below the
  bound. The file needs no benchmark machinery, and it already runs in the normal suite. That makes
  it the right tool for a config invariant.
- **Use fixtures, never the live SEC API.** The API is non-deterministic and rate-limited, and the
  rate limit is often the thing under test.
- **Where a path is allocation-dominated, allocation count is a usable proxy for CPU cost.** An
  allocation-count assertion in a plain `#[test]` is cheap, deterministic, and cross-platform. It
  needs no bench job, no hosted service, and no Valgrind, so weigh it against a real benchmark
  framework. Profile the path first, to confirm that allocation dominates it.

## Open decisions

1. **Does the project need a framework at all, or do assertions in ordinary tests suffice?** Answer
   this question first. The answer can make decisions 2 to 4 unnecessary.
2. **Which framework, if the project needs one.** The team surveyed three and chose none. `criterion`
   is mature, uses wall-clock, and stores baselines. `divan` is easier to use and has weaker
   baselines. `iai-callgrind` gives deterministic instruction counts, needs Valgrind, and **does not
   run on Apple Silicon**.
3. **Where the baseline lives.** The team surveyed two options. CodSpeed and Bencher give hosted
   history, pull request comments, and thresholds, but a third-party service then holds internal
   numbers. `critcmp` with a `gh-pages`-style store adds no external dependency, and we own the
   comparison logic and the storage.
4. **What the project measures.** Wall-clock, instruction counts, or allocation counts. This choice
   drives decisions 2 and 3 more than any other.
5. **Trigger and threshold.** Decide when a benchmark runs, and what counts as a regression. The
   options are a path-filtered pull request job, an opt-in label, or a weekly cron. A deterministic
   measurement tolerates a tight bound. Wall-clock does not.

## Rollout phasing

Follow this sequence and do not skip ahead.

| Phase | What | Status |
| --- | --- | --- |
| 0 | The skill and its references hold the performance knowledge | In progress |
| a | Profile real components along real axes, on demand | Available for CPU |
| b | Show results to CI and to review, with nothing enforced | Not started |
| c | Enforce CI gates against a chosen baseline | Not started. The team can decide never to build it |

Phase b makes results visible. A pull request can never fail on them. Any gate belongs to phase c,
which needs an explicit decision. Phase c also needs an override for a deliberate change. One such
change is a new `MIN_REQUEST_INTERVAL`, which is a config move rather than a regression.
