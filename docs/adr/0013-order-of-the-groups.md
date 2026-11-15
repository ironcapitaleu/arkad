# ADR 0013: The Groups of the Roll-Up Tree Follow the Order of the Design Tables

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([STA-521 comment `bd91a832`](https://linear.app/state-machine/issue/STA-521#comment-bd91a832)) |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-521](https://linear.app/state-machine/issue/STA-521) acts on the decision. [STA-520](https://linear.app/state-machine/issue/STA-520) follows it for the balance sheet |

## Context

The roll-up tree of `xbrl/DESIGN.md` sums the standard concepts in **groups**. A group is one sum:
a parent and the concepts that add up to it. `statement::groups()` in
`xbrl/src/core/elements/statement/mod.rs` lists the groups. Each statement module holds a `GROUPS`
list, and `groups()` joins them: the balance sheet, then the income statement, then the cash flow
statement.

The derivation from a closed group (slice 4 of the roll-up tree) visits the groups in one order.
The first group that gives a value for a member wins. Every other group that holds the member only
checks it. So the order of `groups()` decides which group gives a derived value.

Two orders can fill a `GROUPS` list:

- **Table order**: the order in which each group first appears in the table of its statement in
  `xbrl/DESIGN.md`, "Roll-Up Tree". A group appears with the row of its first child.
- **Parent order**: the order of the parents of the groups in `Concept::ALL`. `Ord` on
  `CanonicalElementKind` sorts by this order.

On `main` at `a9d57d1`, `groups()` lists 10 groups, and the two orders agree. The rustdoc of
`groups()` states parent order, and a test sorts the parents by `Ord`.

With the 7 new income statement and cash flow concepts, the income statement lists 7 groups, and
the two orders differ:

- Table order puts `operating-income` before `operating-expenses`. Parent order puts
  `operating-expenses` (parent `OperatingExpenses`) before `operating-income` (parent
  `OperatingIncome`), because `OperatingExpenses` comes before `OperatingIncome` in the table.
- `net-income` and `net-income-attribution` have the same parent, `NetIncome`. Parent order does
  not tell which comes first.

The balance sheet has the same conflict with the 9 new balance sheet concepts.

The choice binds slice 4, the checks of slice 5, and every later group.

## Decision

1. **Each statement lists its groups in table order.** A group takes the place of the first row of
   the table of its statement in `xbrl/DESIGN.md` that names it in the column "Group". `groups()`
   keeps the order of the statements: balance sheet, income statement, cash flow statement.
2. **The derivation from a closed group visits the groups in the order of `groups()`.** So the
   order in `xbrl/DESIGN.md`, "Derivation From a Closed Group", and the order of the code are one
   order.

With the 7 new concepts, the order is:

- Income statement: `gross-profit`, `operating-income`, `operating-expenses`, `pretax-income`,
  `net-income`, `net-income-attribution`, `comprehensive-income`.
- Cash flow statement: `net-change-in-cash`, `operating-cash-flow`, `investing-cash-flow`,
  `financing-cash-flow`.

## Options Considered

Read on 2026-11-15, on `main` at `a9d57d1`, from the tables of `xbrl/DESIGN.md`, "Roll-Up Tree".

| Option | Order of `operating-income` and `operating-expenses` | Order of `net-income` and `net-income-attribution` | Source of the order |
| ------ | ---------------------------------------------------- | -------------------------------------------------- | ------------------- |
| **A: table order (chosen)** | `operating-income` first | `net-income` first | the design tables, which the derivation already reads |
| B: parent order | `operating-expenses` first | undefined: one parent | the order of the variants in `Concept::ALL` |

- A gives one order for every pair of groups, the same order that "Derivation From a Closed Group"
  states. It follows the cascade of the income statement from top to bottom.
- B needs a second rule for two groups with one parent. It ties the order of the groups to the
  order of the variants, so a new variant can move a group.

## Consequences

- The rustdoc of `groups()` and of each `GROUPS` states table order and links this ADR.
- The test that sorts the parents by `Ord` goes. The test that writes out the names of `groups()`
  in order guards the order.
- If a table of `xbrl/DESIGN.md` changes its rows, the `GROUPS` list of that statement changes in
  the same PR.
- A new group takes its place by the first row that names it, with no change to the other groups.
- The decision does not cover the order of the members inside a group, or the order of the
  concepts in a finding. `xbrl/DESIGN.md`, "Names in Silver", holds the order of the concepts in a
  finding.

## Open Questions

- None today. A group that one table cannot hold, such as a group across two statements, reopens
  the decision.

## Sources

- STA-521 description, "Decided: the order of the groups follows the tables of `xbrl/DESIGN.md`"
  (the project lead, 2026-11-15).
- `xbrl/DESIGN.md`, "Income Statement", "Cash Flow Statement", and "Derivation From a Closed
  Group", on `main` at `a9d57d1`.
- `xbrl/src/core/elements/statement/mod.rs` on `main` at `a9d57d1`: the rustdoc of `groups()` and
  the test `should_match_the_order_of_concepts_when_listing_the_parents_of_the_groups`.
