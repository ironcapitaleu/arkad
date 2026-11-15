# ADR 0019: The Placement of a Child Records Zero if Absent

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-15 ([STA-523 comment `6b5434e9`](https://linear.app/state-machine/issue/STA-523#comment-6b5434e9)) |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-523](https://linear.app/state-machine/issue/STA-523) acts on the decision. [STA-522](https://linear.app/state-machine/issue/STA-522) follows it for the checks |

## Context

The roll-up tree of `xbrl/DESIGN.md` sums the standard concepts in **groups**. A group is
**closed** if its members equal the parent by definition, such as `Assets` = `CurrentAssets` +
`NonCurrentAssets`. A child of a closed group is **zero if absent** if the group uses zero for it
when the series holds no value for it at a period.

Four children are zero if absent: `TemporaryEquity`, `NoncontrollingInterest`,
`NetIncomeAttributableToNoncontrollingInterest`, and `ExchangeRateEffect`. Most filers have none of
these items and report no zero for them.

The tree must record which children are zero if absent. Two rules read it:

- the derivation from a closed group, `core/roll-up/<group>@1`
- the checks `RollUpMismatch` and `NegativeOther`, which are not built yet

Three places can hold the flag:

- **A field of `Placement::Child`**, next to the group and the weight of the child.
- **A field of `Group`**: a list of the children of the group that are zero if absent.
- **A list in each statement module**, next to `GROUPS`.

`Placed` is a trait that other code implements, so its signature stays as it is (`AGENTS.md`,
"Open-Closed Principle"). `Placement` is a `#[non_exhaustive]` enum, and `Group` is a
`#[non_exhaustive]` struct. A new field of either is no change to a trait.

## Decision

1. **`Placement::Child` has the field `zero_if_absent`.** `Placement::child` sets it to `false`,
   and `Placement::child_zero_if_absent` sets it to `true`. `Placement::is_zero_if_absent` reads it.
2. **The tree records zero if absent for the check.** `ExchangeRateEffect` is zero if absent in the
   tree. The derivation treats it as unknown when it has no value, and holds this exception in its
   own module. The checks read the flag of the tree as it is.

## Options Considered

Read on 2026-11-15, on `main` at `97dd816`.

| Option | Where a reader finds the flag | Can the flag name a concept outside the group | Lists to keep in step |
| ------ | ----------------------------- | --------------------------------------------- | --------------------- |
| **A: field of `Placement::Child` (chosen)** | in the placement of the concept, with its group and weight | no | 1: the placements |
| B: field of `Group` | in the `GROUPS` list of the statement | yes, a test must guard it | 2: the placements and `GROUPS` |
| C: list per statement module | in a third list of the statement module | yes, a test must guard it | 3: the placements, `GROUPS`, and the list |

- A keeps every fact about one child in one place, as the column "Zero if absent" of the tables in
  `xbrl/DESIGN.md` does for each row.
- B and C repeat the membership of a group. A child can then be zero if absent in a group it does
  not belong to, and only a test catches it.

## Consequences

- Each statement module sets the flag in its `Placed` impl. A test in
  `xbrl/src/core/elements/statement/mod.rs` pins the four concepts.
- `Placement` derives `Ord` and `Hash`, so two placements that differ only in the flag are not
  equal. A comparison of placements compares the flag too.
- A crate outside `xbrl` that matches `Placement::Child` already uses `..`, because the variant is
  `#[non_exhaustive]`. No caller changes.
- The checks use zero for `ExchangeRateEffect` when it has no value, as Decision 2 says. How
  `RollUpMismatch` and `NegativeOther` report that zero is for the ticket of the checks.

## Open Questions

- None today. A second exception like `ExchangeRateEffect`, a child that is zero if absent in the
  derivation and not in the check, or a flag with a third value, reopens the decision.

## Sources

- STA-523 description, "Additional Notes": the three options and the rule on `Placed` (2026-11-15).
- `xbrl/DESIGN.md`, "Groups" and "Derivation From a Closed Group", on `main` at `97dd816`.
- `xbrl/src/core/elements/roll_up.rs` on `main` at `97dd816`: `Placement` and `Group`.
