# ADR 0014: Errors Nest by Origin, as Value Types

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([ironcapitaleu/arkad#561](https://github.com/ironcapitaleu/arkad/pull/561)) |
| Date | 2025-06-04 |
| Deciders | The project lead |
| Ticket | None. The decision is older than the Linear tickets: the first ticket in a commit message is STA-130, on 2026-08-10 |

## Context

A pipeline of the `sec` crate fails in a state, in a transition between states, or in the setup of
the state machine (ADR 0013). A caller needs one error type to propagate, and a test needs the exact
cause to compare.

The `sec` crate holds an error hierarchy in `sec/src/lib/error/` since #23 (2025-06-04). This ADR
records that decision. It is written after the fact. No document from 2025 records why this shape
won. `sec/design/uml_class_diagram/sec_error_handling.md` draws the types and states no reason.

Other decisions already rest on this one:

- ADR 0006 makes the `storage` errors "mirror the `sec` error hierarchy".
- ADR 0011 nests the standard concepts "as the `sec` errors nest by level".
- `xbrl/src/error/` copies the nesting, `From`, and `TryFrom` (`xbrl/src/error/mod.rs:31-40`).

A **value type** in this ADR is a type that derives `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`,
and `Hash`, as ADR 0006 defines it.

## Decision

1. **Errors nest by origin, one enum per level.** `ErrorKind` holds `StateMachine`. `StateMachine`
   holds `State` and `Transition`. These hold the leaf errors, one module per leaf error. Each
   enum is `#[non_exhaustive]` (`sec/src/lib/error/mod.rs:30-46`,
   `sec/src/lib/error/state_machine/mod.rs:32-55`).
2. **Each level converts up with `From` and down with `TryFrom`.** A `TryFrom` that does not match
   returns `ErrorKind::DowncastNotPossible` (`sec/src/lib/error/mod.rs:44-104`). A caller propagates
   one top type and still recovers the specific cause (`sec/src/lib/error/mod.rs:5-8`).
3. **Every error is a value type.** No error holds a `Box<dyn Error>`, so a test compares a whole
   error with one `assert_eq!`, as `AGENTS.md`, "Testing", requires.

The text of an error follows the rules of `AGENTS.md`, "Error Naming Conventions" and "Error Display
Format": the `[Name]` prefix, `Caused by:` and `Reason:`, and pass-through layers. Those rules set
how code is written, so `AGENTS.md` is their record, not this ADR.

## Options Considered

No record names the options that were weighed in 2025. The repository mentions neither `anyhow` nor
one flat error enum for `sec`.

ADR 0006 weighed options later, for the `storage` errors, on 2026-08-10. A boxed `source()` lost
`PartialEq`, `Clone`, `Eq`, and `Hash`, so no `sec` error could hold it. That comparison is the only
written one, and it covers `storage` only.

## Consequences

- A test asserts on the whole error value with one `assert_eq!`.
- A new error adds one module and one variant at its level, and its siblings do not change.
- The hierarchy costs modules: 19 `mod.rs` files under `sec/src/lib/error/` on `main` at `a9d57d1`.
- A third-party error that is no value type, such as `reqwest::Error`, becomes a reason string at
  the boundary, as ADR 0006 states for `storage`.
- A caller that walks `source()` meets a pass-through layer in place of its inner error, so a
  downcast of `source()` to the inner type does not match (`AGENTS.md`, "Error Display Format").
- The `xbrl` errors are no value types. They derive only `Debug`, `PartialEq`, and `Error`
  (`xbrl/src/error/mod.rs:30`, `xbrl/src/error/validation/mod.rs:16`), so they break decision 3.
- The `api` crate does not follow this shape. Its `ApiError` is one flat enum with `From` impls
  (`api/src/error/mod.rs:28`). This ADR does not cover `api`.

## Sources

- `sec/src/lib/error/mod.rs:5-104` and `sec/src/lib/error/state_machine/mod.rs:32-95`, on `main` at
  `a9d57d1`.
- `xbrl/src/error/mod.rs:31-40` and `api/src/error/mod.rs:28`, on `main` at `a9d57d1`.
- `git log`: `d3b766b` (2025-06-04, #23, "Refactor/sec package") adds `sec/src/lib/error/` with
  `DowncastNotPossible` and the derives. `b3282ad` (2026-06-18, #93) adds the Display convention to
  `AGENTS.md`. `87aa21e` (2026-11-06, #418) adds the pass-through rule.
- ADR 0006, "Context" and "Options Considered". ADR 0011, "Context" and "Decision".
