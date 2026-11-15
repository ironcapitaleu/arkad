# ADR 0013: The Pipelines Run on a Typestate Hierarchical State Machine

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2024-09-03 |
| Deciders | The project lead |
| Ticket | None. The decision is older than the Linear tickets: the first ticket in a commit message is STA-130, on 2026-08-10 |

## Context

The `sec` crate runs its pipelines as a chain of steps: validate a CIK, prepare a request, execute
it, store the result. Each step takes the output of the step before it. The `api` and `xbrl` crates
do not run steps.

The workspace holds a framework crate for these steps, `state_maschine`. It defines the traits
`StateMachine`, `State`, `StateData`, `Context`, `Transition`, and `SuperState`
(`state_maschine/src/state_machine/`). This ADR records the decision to build the pipelines on it.
It is written after the fact. No document from 2024 records why the framework won, or which other
shapes were weighed.

Terms used here:

- A **state** is one step. It holds its input data, computes its output data, and keeps a context
  that outlives one input/output cycle (`state/context/mod.rs:3-6`).
- A **super-state** is a state that holds a state machine of inner states, such as
  `ExtractCompanyFactsSuperState`.
- **Typestate** means that the type of a value encodes its current state, so the compiler rejects a
  move that the design does not allow.

The choice binds:

- every pipeline of `sec`: 21 `State` impls, 24 `Transition` impls, and 7 super-states on `main` at
  `a9d57d1` (counted with `rg` on 2026-11-15, test code left out),
- the binaries `backfill`, `daily_filings`, `extraction`, and `stream_etl`,
- the skills `state-create`, `state-design`, `state-implementation`, `state-scaffold`,
  `transition-create`, `transition-design`, and `transition-implementation`.

## Decision

1. **Every pipeline step is a state of a state machine.** A step implements `State`: input data,
   `compute_output_data`, output data, and a context.
2. **A step with sub-steps is a super-state.** `SuperState<S>` is a state machine and a state at
   once (`super_state/mod.rs:19`). It is generic over its current inner state, so its type changes
   with each inner step.
3. **A transition consumes the machine and returns a machine of another type.**
   `Transition<T, U>::transition_to_next_state(self)` takes `self` by value and returns
   `Self::NewStateMachine` (`transition/mod.rs:20-31`). A move that no `Transition` impl allows does
   not compile, and the old state cannot be used again (`transition/mod.rs:5-6`).
4. **The framework is a crate of its own, with no domain code.** `state_maschine` knows no SEC
   type. `sec` adds its own `State` trait on top, with an async `compute_output_data_async` that
   returns a `StateError`, because its states do network I/O
   (`sec/src/lib/traits/state_machine/state/mod.rs:28-40`, added in #23 on 2025-06-04).

## Options Considered

No record names the options that were weighed in 2024. The repository mentions no other state
machine crate and no plain pipeline of async functions as an alternative. This ADR records the
decision and its consequences only, and does not reconstruct a comparison.

## Consequences

- A wrong order of steps fails at compile time, not at run time.
- Each step costs a fixed set of types: the state, its input data, its output data, its context,
  and the transition into it. The `state-scaffold` skill exists to "Generate the complete
  boilerplate for a new state" (`.claude/skills/state-scaffold/SKILL.md:16`).
- A super-state carries a type parameter per injected dependency and one for its current state.
  `ExtractFilingSuperState<S, X, I, L, C>` has five (`filing/mod.rs:356`).
- The core `State::compute_output_data` is synchronous and returns no error. Every async or
  fallible step goes through the `sec` trait of decision 4.
- `Transition` returns its error as a `&'static str` (`transition/mod.rs:30`), so a failed transition
  carries no typed cause.
- Only `sec` depends on the framework. The `api` and `xbrl` crates do not use it.

## Open Questions

- Whether `Transition` returns a typed error instead of `&'static str`. A change touches the trait
  and its 24 impls in `sec`, so it needs its own decision.

## Sources

- `state_maschine/src/state_machine/`: `mod.rs:28-39`, `state/mod.rs:35-67`,
  `state/context/mod.rs:3-35`, `transition/mod.rs:5-31`, `super_state/mod.rs:19`, on `main` at
  `a9d57d1`.
- `sec/src/lib/traits/state_machine/state/mod.rs:28-40` and
  `sec/src/lib/implementations/states/extract/filing/mod.rs:356`, on `main` at `a9d57d1`.
- `git log`: `e33b0e6` (2024-09-03, "init: setup workspace") adds the framework. `2b58dac`
  (2024-09-03, #3) makes it a dependency of `sec`. `26b1d58` (2024-09-08, #6) adds the first
  super-state. `cdff125` (2025-04-29, #19) adds the first working state.
- `rg` counts of `SMState`, `SMTransition`, and `SMSuperState` impls in `sec/src`, test code left
  out, on 2026-11-15.
