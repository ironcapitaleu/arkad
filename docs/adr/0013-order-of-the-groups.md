# ADR 0013: The Groups of Each Statement Follow the Order of the Statement

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([STA-520 comment `9ef22314`](https://linear.app/state-machine/issue/STA-520#comment-9ef22314)) |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-521](https://linear.app/state-machine/issue/STA-521/feature-the-7-new-income-statement-and-cash-flow-concepts-of-the-roll) records the decision. [STA-520](https://linear.app/state-machine/issue/STA-520/feature-the-9-new-balance-sheet-concepts-of-the-roll-up-tree) and STA-521 act on it |

## Context

A **group** of the roll-up tree is one sum: a parent and the concepts that add up to it. Each
statement module of `xbrl/src/core/elements/statement/` lists its groups in `GROUPS`.
`statement::groups()` joins the lists: the balance sheet, then the income statement, then the cash
flow statement.

The order of the list is a rule, not a detail:

- The derivation from a closed group (slice 4 of the roll-up tree) visits the groups in this order
  in each pass. The first group that gives a value for a concept wins. A later group that holds the
  same concept only checks it.
- Two rules give an order, and they differ once the new concepts join the tree:
  - **Table order:** each group comes at the place of its first child in the statement, from the
    top line down. The tables of `xbrl/DESIGN.md`, "Roll-Up Tree", list the concepts in this order.
  - **Parent order:** the groups sort by their parent in `Concept::ALL`, which is the `Ord` of
    `CanonicalElementKind`.
- For the 10 groups on `main` at `a9d57d1` (2026-11-15), the two orders agree. The code documents
  parent order, and a test sorts the parents by `Ord`.

The orders differ in three places:

| Statement | Table order | Parent order |
| --------- | ----------- | ------------ |
| Balance sheet | `current-liabilities` before `equity` | `equity` (parent `Equity`) before `current-liabilities` (parent `CurrentLiabilities`) |
| Income statement | `operating-income` before `operating-expenses` | `operating-expenses` (parent `OperatingExpenses`) before `operating-income` (parent `OperatingIncome`) |
| Income statement | `net-income` before `net-income-attribution` | no order: both groups have the parent `NetIncome` |

## Decision

1. **Each statement lists its groups in table order.** A group comes at the row of its first child
   in the statement, from the top line down. The project lead chose this order on 2026-11-15.
2. **The test of the order writes the names out.** No test derives the order from `Ord`. A new
   group takes its place by hand, at the row of its first child.

## Options Considered

Compared on 2026-11-15, on `main` at `a9d57d1` and the concepts of slices 2 and 3.

| Option | Follows the cascade of the statement | Orders two groups with one parent | Test without a written list |
| ------ | ------------------------------------ | --------------------------------- | --------------------------- |
| **A: table order (chosen)** | yes | yes | no: the test writes the 20 names out |
| B: parent order | no: two pairs of groups swap places | no: `net-income` and `net-income-attribution` tie | yes, sort by `Ord` |

- A reads like the statement. A person who checks a derived value walks the statement from the top
  and meets the groups in the order the rule visits them.
- A orders `net-income` and `net-income-attribution`. B gives no order for two groups with one
  parent, so the winner of a derivation would depend on the place of the group in the source.
- B needs no written list, because a sort gives it. That gain is small: each statement has at most
  9 groups.

## Consequences

- `statement::groups()` and each `GROUPS` document the table order. The test that sorted the parents
  by `Ord` is gone. The test of the names in order is the guard.
- Slice 4 visits the groups in table order. If a concept is in two closed groups, the group with the
  first child higher in the statement gives its value.
- The order of the members in a `RollUpMismatch` finding is a separate rule. `xbrl/DESIGN.md`,
  "Names in Silver", holds it.

## Open Questions

- None. A new statement or a new group follows the same rule.

## Sources

- STA-521 description, "Decided: the order of the groups follows the tables" (2026-11-15).
- `xbrl/DESIGN.md`, "Balance Sheet", "Income Statement", "Cash Flow Statement", and "Derivation
  From a Closed Group", on `main` at `a9d57d1`.
- `xbrl/src/core/elements/statement/mod.rs`, test
  `should_match_the_order_of_concepts_when_listing_the_parents_of_the_groups`, on `main` at
  `a9d57d1`.
