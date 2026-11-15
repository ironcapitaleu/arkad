# ADR 0015: Errors Nest by Origin, as Value Types, and Print One Fact per Segment

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([ironcapitaleu/arkad#561](https://github.com/ironcapitaleu/arkad/pull/561)) |
| Date | 2025-06-04 |
| Deciders | The project lead |
| Ticket | None. The decision is older than the Linear tickets: the first ticket in a commit message is STA-130, on 2026-07-20 (#114) |

## Context

A pipeline of the `sec` crate fails in a state, in a transition between states, or in the setup of
the state machine (ADR 0014). A caller needs one error type to propagate, and a test needs the exact
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
4. **An error prints its chain on one line, one segment per layer.** Since #81 (2026-04-26):
   - Each segment starts with the name of its error type in brackets, such as `[StateError]`.
   - A segment that wraps an inner error ends with `Caused by:` and the inner error.
   - A leaf ends with `Reason: '…'`. Since #93 (2026-06-18), a leaf leaves it out if its fields
     already say the cause.
   - Commas separate the parts. No inner error sits inside quotes.

   The reader is an operator. The project lead gave the reason on 2026-11-15. An operator greps a
   log for the type name of a layer. An operator also reads the whole chain in one log line, from
   the outermost layer to the root cause. #416 (2026-11-06, STA-452) fixed a segment with no name
   that hid its inner error in quotes. STA-452 found it in a `backfill` run that filled the disk.
5. **`Display` never uses `Debug` formatting.** Since #84 (2026-05-05). `Debug` output is internal
   to Rust and can change between versions, so code that reads an error text cannot parse it. A
   field that is a collection gets a newtype with its own `Display`.
6. **A layer prints a segment only if it adds a fact.** Since #418 (2026-11-06, STA-454). A fact
   is an action, an object, a cause, or an ID. A layer that adds no fact is `#[error(transparent)]`
   and prints its inner error in its place. On `main` at `a9d57d1`, 18 variants in
   `sec/src/lib/error/` pass through, such as `ErrorKind::StateMachine`
   (`sec/src/lib/error/mod.rs:40`).

   The goal of STA-454: an operator reads what failed, on what, and why, in as few segments as
   possible. Before the rule, a disk-full write printed 7 segments. Five said only that something
   failed, and the state name appeared twice. After it, the chain has 3 segments.

   The project lead confirmed on 2026-11-06 that `source()` of a pass-through layer returns the
   source of its inner error, not the inner error. Otherwise a caller that walks `source()` prints
   the inner segment twice.

`AGENTS.md`, "Error Naming Conventions" and "Error Display Format", holds the detailed rules and
examples for decisions 4 to 6.

## Options Considered

No record names the options that were weighed for decisions 1 to 6. The repository mentions
neither `anyhow` nor one flat error enum for `sec`.

ADR 0006 weighed options later, for the `storage` errors, on 2026-08-10. A boxed `source()` lost
`PartialEq`, `Clone`, `Eq`, and `Hash`, so no `sec` error could hold it. That comparison is the only
written one, and it covers `storage` only.

## Consequences

- A test asserts on the whole error value with one `assert_eq!`.
- A new leaf error adds one module and one variant at its level, and its siblings do not change.
- The hierarchy costs modules: 19 `mod.rs` files under `sec/src/lib/error/` on `main` at `a9d57d1`.
- A third-party error that is no value type, such as `reqwest::Error`, becomes a reason string at
  the boundary, as ADR 0006 states for `storage`.
- A caller that walks `source()` meets a pass-through layer in place of its inner error, so a
  downcast of `source()` to the inner type does not match (`AGENTS.md`, "Error Display Format").
- The `xbrl` errors are not value types. They derive only `Debug`, `PartialEq`, and `Error`
  (`xbrl/src/error/mod.rs:30`, `xbrl/src/error/validation/mod.rs:16`), so they break decision 3.
- The `api` crate does not follow this shape. Its `ApiError` is one flat enum with `From` impls
  (`api/src/error/mod.rs:28`). This ADR does not cover `api`.

## Open Questions

- Whether an error gets a hint on how to fix it, and where. STA-454 compared a `Hint: '…'` segment
  on the leaf error with a `hint` field in the JSON log line, set by the binary. On 2026-11-06 the
  project lead deferred both. Today only two of the four variants of `InvalidRunDates` in
  `daily_filings` name their fix in their text, `MissingStartDate` and `InvalidDate`
  (`sec/src/bin/daily_filings/dates.rs:27-36`).

## Sources

- `sec/src/lib/error/mod.rs:5-104` and `sec/src/lib/error/state_machine/mod.rs:32-95`, on `main` at
  `a9d57d1`.
- `xbrl/src/error/mod.rs:31-40` and `api/src/error/mod.rs:28`, on `main` at `a9d57d1`.
- `git log`: `d3b766b` (2025-06-04, #23, "Refactor/sec package") adds `sec/src/lib/error/` with
  `DowncastNotPossible` and the derives. `dd046c2` (2026-04-26, #81) adds the chain format.
  `bcf5316` (2026-05-05, #84) forbids `Debug` in `Display`. `b3282ad` (2026-06-18, #93) moves the
  rules to `AGENTS.md`. `87aa21e` (2026-11-06, #418) adds the pass-through rule, and `3426b79`
  (2026-11-06, #420) applies it to the state and transition errors of `sec`.
- STA-452 and ironcapitaleu/arkad#416: the unnamed, quoted segment of the disk-full run.
- STA-454, its description and the comments of the project lead of 2026-11-06 (`source()` of a
  pass-through layer, hints deferred), and ironcapitaleu/arkad#418: the 7-segment chain and its goal.
- The reason for decision 4, stated by the project lead on 2026-11-15 and quoted in the
  description of ironcapitaleu/arkad#561.
- ADR 0006, "Context" and "Options Considered". ADR 0011, "Context" and "Decision".
