# ADR 0033: The Byte Form of `content_digest` Prints Each Value as Text With a Length Prefix

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-21 ([STA-463 comment `5fa1a8f7`](https://linear.app/state-machine/issue/STA-463#comment-5fa1a8f7)) |
| Date | 2026-11-16 |
| Deciders | The project owner |
| Ticket | [STA-463](https://linear.app/state-machine/issue/STA-463/design-silver-writer-and-transform-job-plan) |

## Context

Each `silver_files` row holds a `content_digest`. Integrity check 4 runs the transform again on the
same input with the same `transform_version`, and compares the digests. Silver stores the digest of
every unit for good, so a change of the byte form makes every stored digest unequal to a new one.

The silver contract fixes the frame:

- A digest group is one filing, or the entity or the taxonomy release for rows without a filing.
- A digest group hash is the SHA-256 over the rows of the group, sorted by the row key.
- `content_digest` is the SHA-256 over the group IDs and group hashes, sorted by group ID.
- Each row enters in a fixed byte form, without `execution_id`, `loaded_at`, and `computed_at`.
- The transform code prints the values, not the Arrow crate, so an Arrow upgrade keeps the digest.

The owner decided on 2026-11-08 that the digest does not depend on the split or the parts. The
prototype gave five digests for the same 8,634,541 rows of `2024q1`, one per split and bucket count
(Linear document "STA-424 Findings: Silver Parquet Layer and Polars for the Transform Step",
section 8, finding 3).

The column `quarters` changed from `uint8` to `uint16` on 2026-11-08, after `qtrs` reached 4,007 in
`2024q1` (same document, finding 1). A column type can change again.

This ADR fixes the byte form of one value, one row, and the frame around them.

## Decision

1. **Every value starts with a marker byte:** `0x00` for null, `0x01` for a value. A null has no
   more bytes.
2. **A scalar prints as UTF-8 text, then enters as a 4-byte big-endian length and the text.**

   | Arrow type | Text | Example |
   | ---------- | ---- | ------- |
   | `utf8` | The string as stored | `us-gaap` |
   | `bool` | `true` or `false` | `true` |
   | Every integer type | Base 10, no leading zero, `-` before a negative value | `4007` |
   | `decimal128` | Base 10, no leading zero before the point, `0` before the point if the integer part is zero. A `.` and the fraction only if the fraction is not zero, with no trailing zero. No `-0` | `-0.25`, `1234.5`, `100` |
   | `date32` | `YYYY-MM-DD` | `2026-11-06` |
   | `timestamp[ms, UTC]` | `YYYY-MM-DDTHH:MM:SS.mmmZ` | `2026-11-06T13:05:00.000Z` |

3. **A list enters as its marker, a 4-byte big-endian element count, and each element** in its own
   byte form.
4. **A struct enters as its marker, a 4-byte big-endian field count, and each field** in the order
   of the schema.
5. **A row is its columns in the order of the schema of its dataset,** without `execution_id`,
   `loaded_at`, and `computed_at`.
6. **Rows of a group sort by the bytes of their row key**, and then by the bytes of the whole row.
   The group hash is the SHA-256 over the sorted rows, one after the other.
7. **A group ID is the byte form of its key columns,** for example `filing_id`, or
   `entity_scheme` then `entity_id`. Groups sort by these bytes.
   `content_digest` is the SHA-256 over each group ID followed by its 32 hash bytes, printed as 64
   lowercase hex characters. A unit with no rows has the SHA-256 of no bytes.
8. **A golden test pins the form to `transform_version`.** It holds one fixture unit per dataset
   and the expected digest for each `transform_version`. If the digest of the fixtures changes and
   the version does not, the test fails.

## Options Considered

Compared on 2026-11-16, on the facts in "Context".

| Option | Arrow upgrade | Wider column type | Two values with one byte form |
| ------ | ------------- | ----------------- | ----------------------------- |
| **A (chosen). Text per value, length prefix, marker byte** | No change | No change | None. The length and the marker frame each value |
| B. Fixed-width binary per Arrow type, little-endian | No change | New digest | None |
| C. The Arrow IPC bytes of the rows | New digest if the IPC layout changes | New digest | None |
| D. One JSON line per row through `serde_json` | No change | No change | The number text depends on `serde_json` and on its `arbitrary_precision` feature, which `xbrl` turns on |

- A keeps the digest when a column widens, as `quarters` did, and when the scale of a decimal
  changes, because the text has no trailing zero.
- B gives a new digest for every unit of a dataset when one integer column widens.
- C breaks the rule of the contract that the Arrow crate prints no value.
- D leaves the print of numbers and the escape of strings to a crate.

## Consequences

- The same rows give the same digest across splits, parts, Arrow releases, and integer widths.
- A change of the column order or of a print rule changes every digest. It needs a new
  `transform_version`, and the golden test enforces this.
- A new column changes every digest of its dataset. It comes with a new `schema_version`, and so
  with a new `transform_version`.
- The writer prints each value once more for the digest. The cost is not measured.

## Open Questions

- The cost of the print and the hash per Notes file. The feature of the digest measures it on
  `2024q1`.

## Sources

- Linear document "STA-424 Findings: Silver Parquet Layer and Polars for the Transform Step",
  2026-11-08, section 8, findings 1 and 3, and the decisions of the owner of 2026-11-08 in it.
- Linear document "Silver Prototype Findings", 2026-11-09, "Notes for the Silver Writer": the
  `arbitrary_precision` feature of `serde_json`.
- [STA-425](https://linear.app/state-machine/issue/STA-425): the silver contract of
  `content_digest`, read on 2026-11-16.
- [STA-463](https://linear.app/state-machine/issue/STA-463/design-silver-writer-and-transform-job-plan),
  Definition of Done item "The byte form of `content_digest`", read on 2026-11-16.
