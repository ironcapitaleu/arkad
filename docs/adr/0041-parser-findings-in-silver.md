# ADR 0041: Parser Findings in Silver

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-21 ([STA-497 comment `1f6e81fa`](https://linear.app/state-machine/issue/STA-497#comment-1f6e81fa)) |
| Date | 2026-11-21 |
| Deciders | The project lead |
| Ticket | [STA-497](https://linear.app/state-machine/issue/STA-497) |

## Context

The parsers of `xbrl` return `Finding` values next to their facts. Two variants come from the
parser alone:

- `UnmappedRow { filing, row, reason }`: the parser could not map a row into a fact, so it
  dropped the row. `row` holds the table and the primary key of a Notes row, or the element name
  and the context ID of an instance fact. `reason` is an
  `InvalidValue`, the rule the row breaks.
- `UndeclaredFamily { filing, family }`: the filing holds facts of a namespace family that no
  plug-in declares. The parser keeps the facts. They do not join between sources or across
  releases until a plug-in declares the family.

The silver contract had no home for either. Its `findings` dataset had nine kinds, all from the
standardize steps. The fixture filing of the silver prototype of 2026-11-09 gave neither finding.

Integrity check 3 of silver compares row counts inside a unit of `as_is_facts` or `filings`:
`source_rows` minus `dropped_rows` must equal `row_count`. An unmapped row is a source row with no
silver row. If `dropped_rows` does not count it, check 3 fails on every unit with an unmapped row.

The choice binds the row types of `findings` and `silver_files` and their schema snapshots. A
snapshot fixes the columns, and a later change needs a new `schema_version`. No silver data exists
on 2026-11-21, so a reversal today costs no migration.

## Decision

1. **`UndeclaredFamily` and `UnmappedRow` are kinds of `findings`.** The parse filing step writes
   them as filing rows (`step = filing`), with `filing_id` set. Empty `concepts` and `periods`.
2. **A new nullable column `subject` holds what the finding is about.** For `UndeclaredFamily`,
   the family. For `UnmappedRow`, the row as the parser prints it. Null for the other kinds.
3. **`subject` is part of the key of a finding.** One filing can have many unmapped rows and many
   undeclared families. Without `subject`, their rows share a key, and the sort order of
   `content_digest` is not fixed.
4. **`reason` of an `UnmappedRow` holds the name of the `InvalidValueReason` variant**, for example
   `NotANumber`.
5. **`dropped_rows` of `silver_files` counts each unmapped row** under the reason `UnmappedRow`.
   Check 3 then holds.

## Options Considered

Facts from `xbrl/src/core/finding.rs` and the silver contract on `main` at `7e2c83e`, 2026-11-21.

| Option | Which row was lost | Undeclared family | Check 3 | New columns |
| ------ | ------------------ | ----------------- | ------- | ----------- |
| **A (chosen): kinds of `findings`, count in `dropped_rows`** | A query on `findings` | A query on `findings` | Holds | `subject` |
| B: `UnmappedRow` as a count only, `UndeclaredFamily` a kind | The log only | A query on `findings` | Holds | `subject` |
| C: counts in `dropped_rows`, both findings in the log only | The log only | The log only | Holds | None |

- A keeps every fact of the parser in silver. A reader finds the lost rows with the same reader
  as the other findings.
- B and C keep the row only in the log. A log is not a dataset: a replay does not rebuild it, and
  the checks do not read it.
- C keeps no record of an undeclared family. Such facts do not join, so a reader cannot tell why
  a value is missing in a join.

## Consequences

- `findings` has eleven kinds and the column `subject`. The key of a finding has six parts.
- The series step does not copy the parser kinds. They stay filing rows.
- The row of an instance fact names no unit. Two unmapped facts with the same element and context
  share a key. Rows with the same key sort by the bytes of the whole row
  ([ADR 0033](0033-byte-form-of-content-digest.md), decision 6), so `content_digest` stays fixed.
- An `UnmappedRow` appears twice: as a row of `findings`, and in the count of `dropped_rows`. The
  count serves check 3, the row serves a reader.
- A filing that the parser rejects as a whole is not covered. It gives a parse error, not a
  `Finding`.

## Open Questions

- A new parser `Finding` variant needs its own decision on `subject` and `reason`.

## Sources

- `xbrl/src/core/finding.rs` at `7e2c83e`: the variants `UndeclaredFamily` and `UnmappedRow`.
- `xbrl/src/adapter/edgar/notes/mod.rs` at `7e2c83e`: the parser returns these findings, and a
  `sub` row that breaks a rule fails the filing.
- `xbrl/src/adapter/edgar/instance/mod.rs` at `7e2c83e`: the row of an instance fact is the
  element name and the context ID, for example `us-gaap:Revenues c-1`.
- [ADR 0033](0033-byte-form-of-content-digest.md), decision 6: rows of one key sort by the bytes of
  the whole row.
- Linear document "Silver Prototype Findings", 2026-11-09, gap 5.
