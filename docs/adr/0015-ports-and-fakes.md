# ADR 0015: Every I/O Boundary Is a Trait, Injected, and Faked in Unit Tests

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([ironcapitaleu/arkad#561](https://github.com/ironcapitaleu/arkad/pull/561)) |
| Date | 2026-04-16 |
| Deciders | The project lead |
| Ticket | None. The decision is older than the Linear tickets: the first ticket in a commit message is STA-130, on 2026-08-10 |

## Context

The pipelines of `sec` talk to the outside: the SEC over HTTP, a clock through a rate limiter, and a
store for raw documents. A unit test must run without the network and without a real store.

Since #59 (2026-04-16), each of these boundaries is a trait. That PR turned the concrete `SecClient`
struct of #30 into the traits `SecClient` and `InnerClient`, with a `ReqwestClient` adapter and the
first fakes. This ADR records that decision. It is written after the fact. The commit gives no
reason, and no document compares options.

Terms used here, as `sec/design/data_model/design_patterns_primer.md` uses them:

- A **port** is a trait for one I/O boundary.
- An **adapter** is a type that implements a port with a real system, such as `ReqwestClient`.
- A **fake** is a minimal implementation of a port that returns fixed, predictable responses
  (`.claude/skills/testing/SKILL.md:285-289`).
- A **composition root** is the code that builds the concrete adapters, such as a binary
  (ADR 0005).

ADR 0005 decides that the `storage` crate holds the storage ports with no database driver. This ADR
is the general rule that ADR 0005 applies to storage.

## Decision

1. **Every I/O boundary is a port.** On `main` at `a9d57d1`, the ports are `SecClient`,
   `InnerClient`, and `Executes` (`sec/src/lib/shared/http_client/traits/`), `RateLimiter`,
   `RawRepository`, `DailyClient`, `AllClient`, and the storage ports `WriteRepository`,
   `ReadRepository`, and `ListRepository`.
2. **Code takes the port, and a composition root picks the adapter.** A state, a builder, or a
   function takes a generic parameter or an associated type, such as
   `ExtractCompanyFactsSuperStateContext<R, C = SecClient>`
   (`sec/src/lib/implementations/states/extract/company_facts/mod.rs:125-129`). Library code names
   no concrete adapter.
3. **Unit tests use hand-written fakes, and integration tests use the real adapter.** Fakes live in
   `tests/fixtures/` of each crate, such as `FakeSecClient`, `AlwaysReadyRateLimiter`, and
   `FakeWriteRepository`. Fakes are for unit tests only (`.claude/skills/testing/SKILL.md:349-350`).

The `testing` and `domain-concept` skills hold the rules on how to write a port and a fake, and the
names of fakes. Those rules set how code is written, so the skills are their record.

## Options Considered

No record names the options that were weighed. The repository mentions no mock library, such as
`mockall`. It names no other shape, such as tests against the live SEC or concrete types with test
flags. This ADR records the decision and its consequences only.

## Consequences

- A unit test runs with no network, no file system, and no timer, and its result does not change
  between runs.
- A fake tests the design of a port before any third-party dependency exists
  (`.claude/skills/domain-concept/SKILL.md:229-231`).
- Each port needs fakes, and each fake needs upkeep when the port changes. The Open-Closed rule of
  `AGENTS.md` limits such changes.
- Generic parameters add up. A super-state carries one per injected port (ADR 0013, "Consequences").
- A fake checks only what it was written to return. Integration tests with the real adapter cover
  the rest.
- The `api` and `xbrl` crates have no ports on `main` at `a9d57d1`. `xbrl` has `Taxonomy`, a plug-in
  for taxonomy data, not an I/O boundary.

## Sources

- The trait files named in "Decision", on `main` at `a9d57d1`.
- `sec/src/lib/tests/fixtures/` and `storage/src/tests/fixtures/`, on `main` at `a9d57d1`.
- `git log`: `f3ad724` (2025-07-22, #30) adds `SecClient` as a struct. `33ca542` (2026-04-16, #59,
  "refactor: extract") adds the `SecClient` and `InnerClient` traits and the first fakes.
  `6c52da1` (2026-07-12, #101) adds `RateLimiter`. `7b765e6` (2026-08-17, #138) adds the first
  storage port.
- `.claude/skills/testing/SKILL.md:285-289` and `:349-350`, and
  `.claude/skills/domain-concept/SKILL.md:148-151` and `:229-231`.
- `sec/design/data_model/design_patterns_primer.md`, "Ports & Adapters (Hexagonal Architecture)".
- ADR 0005.
