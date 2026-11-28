# ADR 0053: Duplicate Rows and Row Counts from the Parsers

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-28 ([STA-613 comment `cb0340de`](https://linear.app/state-machine/issue/STA-613#comment-cb0340de), accessed 2026-11-28) |
| Date | 2026-11-28 |
| Deciders | The project lead |
| Ticket | [STA-613](https://linear.app/state-machine/issue/STA-613) (accessed 2026-11-28) |

## Context

The Notes parser and the instance parser of `xbrl` merge rows that repeat a fact. A row repeats a
fact if an earlier row is equal to it in key, value, and `decimals`. `number_copies` in
`xbrl/src/adapter/edgar/as_is.rs` keeps the first row and drops the others. A dropped row gives no
fact and no finding.

Integrity check 3 of silver, "Row counts", compares rows inside a unit of `as_is_facts`:
`source_rows` minus `dropped_rows` must equal `row_count`. A merged row is a source row with no
silver row. If `dropped_rows` does not count it, check 3 fails on every unit with a merged row.
[ADR 0041](0041-parser-findings-in-silver.md) gave the reason `UnmappedRow` and no reason for a
merged row.

The parse states fill `SourceCounts` per filing. They need three numbers: the rows in, the
unmapped rows, and the merged rows. The unmapped rows are the `UnmappedRow` findings. The
parsers reported no count of the other two. The instance parse state holds only the bytes of the
document, so it cannot count the fact elements without a second XML parse.

`dropped_rows` stores the reason as text in `unit_manifests`. A rename after the first run costs a
migration of stored data.

## Decision

1. **The reason of a merged row is `DuplicateRow`.** It has the casing of `UnmappedRow`.
   `DropReason::DuplicateRow` prints it.
2. **The parsers count the merged rows.** `number_copies` returns `duplicate_rows`, and
   `NotesFacts` and `InstanceFacts` carry it.
3. **The parsers count the rows in.** `NotesFacts::rows_in` counts the `num` and `txt` rows.
   `InstanceFacts::rows_in` counts the children of the root with a `contextRef`, as the parser
   reads them.
4. **No count comes from a subtraction.** A parse state copies the counts of the parser into
   `SourceCounts`.

## Options Considered

Facts from `xbrl/src/adapter/edgar/as_is.rs` and both parsers on `main` at `699e653`, 2026-11-28.

| Option | Check 3 can fail | XML parses per instance | Change in `xbrl` |
| ------ | ---------------- | ----------------------- | ---------------- |
| **A (chosen): the parsers count rows in and merged rows** | Yes | 1 | Two fields per parser output |
| B: the state subtracts facts and unmapped rows from rows in | No | 1, with `rows_in` from the parser | One field per parser output |
| C: a separate pass counts the instance fact elements | Yes | 2 | A new function |

- A counts each number where it happens. A row that the parser loses without a count makes check
  3 fail.
- B makes `source_rows - dropped_rows = row_count` true by construction, so check 3 can never
  catch a lost row.
- C counts the elements with the same rule as the parser, so it catches no more than A, and it
  parses the XML twice.

## Consequences

- `dropped_rows` has the reasons `UnmappedRow` and `DuplicateRow`.
- A merged row appears only in the count. It has no row in `findings`, since it lost no fact.
- `DropReason` has no `#[non_exhaustive]` marker. A new reason then fails to compile in each
  `match` that must handle it.
- The instance count of `source_rows` comes from the parser. A parser that skips an element
  without counting it reads it neither as a fact nor as a row in, so check 3 does not catch it.

## Open Questions

- None.

## Sources

- `xbrl/src/adapter/edgar/as_is.rs` at `699e653`: `number_copies` drops a candidate equal to an
  earlier one in key, value, and `decimals`.
- `xbrl/src/adapter/edgar/instance/document.rs` at `699e653`: `Document::facts` returns the
  children of the root with a `contextRef`.
- `sec/design/pipeline/silver_layer.md`, `unit_manifests`, `source_rows` and `dropped_rows`, and
  check 3 "Row counts".
- [ADR 0041](0041-parser-findings-in-silver.md), decision 5: `dropped_rows` counts each unmapped
  row.
