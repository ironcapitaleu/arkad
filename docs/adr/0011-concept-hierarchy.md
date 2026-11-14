# ADR 0011: Standard Concepts Nest by Statement, as the `sec` Errors Nest

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-14 ([STA-500 comment `dd86cbdf`](https://linear.app/state-machine/issue/STA-500#comment-dd86cbdf)) |
| Date | 2026-11-14 |
| Deciders | The project lead |
| Ticket | [STA-500](https://linear.app/state-machine/issue/STA-500) acts on the decision. [STA-499](https://linear.app/state-machine/issue/STA-499) made the design |

## Context

The standard concepts are the target of resolution in the `xbrl` crate. Each alias list, rule, fact,
and finding names one. The `api` crate builds the concept ID and the name of `/v1/concepts` from it.
Silver stores its name in the `concept` column of `standardized_facts`, `candidates`, and `findings`.

The concepts were one flat enum, `CanonicalElement`, with 39 variants and a flag for Level 1. The
roll-up tree of `xbrl/DESIGN.md` gives each concept of a statement one **placement**: a top line,
or a child of one **group** with a weight of +1 or −1. A group is one sum: a parent and the
concepts that add up to it. A concept of a disclosure, such as earnings per share, has no
placement.

The `sec` crate has an error hierarchy in `sec/src/lib/error/`: nested `#[non_exhaustive]` enums,
one module per level, `From` one level up, and `TryFrom` one level down. ADR 0006 copies it for the
storage errors.

The choice binds:

- every use of the type: 372 lines in 18 Rust files on `main` at `a1a3471` (`git grep -c
  CanonicalElement -- '*.rs'`, 2026-11-14),
- the `api` crate, which reads the name and the label of each concept,
- the later work of the roll-up tree: new concepts, the derivation from a closed group, the
  checks, and the statement views.

## Decision

1. **The standard concepts nest by statement and disclosure, as the `sec` errors nest by level.**
   `CanonicalElementKind` holds `StatementConcept` or `DisclosureConcept`. These hold the
   innermost enums: `BalanceSheetConcept`, `IncomeStatementConcept`, `CashFlowConcept`, and
   `PerShareConcept`. Each enum is `#[non_exhaustive]` and has its own module. Each level converts
   up with `From` and down with `TryFrom`.
2. **The nesting stops at the statement.** The parent and the weight of a concept live in
   `Placed::placement()`, and each statement module lists its groups. A parent is a sum, not a
   container, so the type does not nest a child under its parent.
3. **Two traits describe one concept.** Each innermost enum implements `Concept` (`ALL`, `name`,
   `label`). Each enum of a statement also implements `Placed: Concept` (`placement`). A
   disclosure implements `Concept` only.
4. **Two macros define the enums from one list each.** The innermost macro writes the enum,
   `ALL`, `ELEMENTS`, `name()` from `stringify!`, and the inherent `const fn element()`. The middle
   macro writes both middle enums, their conversions, and `INNERMOST`.
5. **The name in silver is the name of the innermost variant**, such as `NetIncome`, not a path.
   `Serialize` writes it by hand, and `FromStr` parses it back. `Display` writes the label.

## Options Considered

No option was measured. The facts below follow from the Rust type rules and from the tables of
`xbrl/DESIGN.md`, "Roll-Up Tree".

| Option | A new statement changes | A move to a new parent changes | The statement of a concept comes from |
| ------ | ----------------------- | ------------------------------ | ------------------------------------- |
| **A (chosen):** nest by statement, placement in a method | one variant of `StatementConcept` and one new module | one arm of `placement()` | the type |
| B: one flat enum with a `statement()` method | an arm of `statement()` for each new concept | one arm of `placement()` | a method that a new variant can miss |
| C: nest by parent, such as `Assets(AssetsConcept)` | the tree types | the type of the concept, and each `match` on it | the type |

- A gives each statement its own module, so a new statement or disclosure adds a module and a
  variant, and its siblings do not change. It follows a hierarchy the team already reads.
- B keeps one large enum. A `match` on the statement is a method, so the compiler cannot tell a
  concept of the balance sheet from a concept of a disclosure.
- C makes a sum a container. A step starts from a line of another statement, as
  `operating-cash-flow` starts from `NetIncome`, and that line cannot sit inside two parents. A
  move to a new parent changes a type and every `match` on it.

## Consequences

- A const table, such as an alias list, writes `BalanceSheetConcept::Assets.element()`. A trait
  method cannot be a `const fn`, so `element()` is an inherent method.
- `Ord` sorts by statement first, then by concept.
- Every stored name stays valid. No concept changes its variant name, and a test checks that no
  two concepts in `INNERMOST` share a name.
- `ConceptId` of the `api` crate reads `CanonicalElementKind::name()`, and `/v1/concepts` returns
  the same body.
- A group is listed only if each of its members is a standard concept. A placement can name a group
  that the list does not hold, such as `liabilities-and-equity`.
- A disclosure whose values add up, such as debt maturities, needs a trait of its own next to
  `Placed`. This decision does not cover it.

## Open Questions

- Whether a disclosure with sums gets a trait of its own, or a placement of its own. The design of
  the first such disclosure answers it.

## Sources

- `xbrl/DESIGN.md`, "Roll-Up Tree", "Extension Model", and "Graph Structure", merged in
  ironcapitaleu/arkad#501.
- `sec/src/lib/error/` on `main` at `a1a3471`.
- ADR 0006, "Storage Errors Mirror the `sec` Error Hierarchy".
- `git grep -c CanonicalElement -- '*.rs'` on `main` at `a1a3471`, 2026-11-14: 372 lines in 18
  files.
