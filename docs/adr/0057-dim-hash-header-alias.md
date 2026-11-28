# ADR 0057: The Header Alias of the Dimension Hash in `dim.tsv`

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-28 ([STA-566 comment `dfd39b27`](https://linear.app/state-machine/issue/STA-566#comment-dfd39b27), accessed 2026-11-28) |
| Date | 2026-11-28 |
| Deciders | The project owner |
| Ticket | [STA-566](https://linear.app/state-machine/issue/STA-566), accessed 2026-11-28 |

## Context

Layer 3 of the Notes reader checks that the header of each table equals the columns of the
metadata member. The metadata member is a CSVW file. It lists the `name` of each column of each
table.

On 2026-11-28, the first column of `dim.tsv` broke this rule in two files:

| File | Header of `dim.tsv` | `name` in the metadata member | `titles` in the metadata member |
| ---- | ------------------- | ----------------------------- | ------------------------------- |
| `2009q1_notes.zip` | `dimhash` | `dimh` | `Dimension Hash` |
| `2026_10_notes.zip` | `dimhash` | `dimh` | `Dimension Hash` |

The headers of the other seven tables equal the `name` of their columns in both files. `num.tsv`
and `txt.tsv` name the same value `dimh` in their headers.

A literal check fails every Notes file. The choice binds the Notes reader of STA-566 and every
reader that maps a column of `dim.tsv` by name.

## Decision

1. **The header check compares each header to the `name` of the columns of the metadata member,
   with one alias compiled into `sec`.** In `dim.tsv`, the metadata name `dimh` matches the header
   `dimhash`.
2. **Every other column must match its `name` exactly.** A new difference fails the file as a bad
   unit, and a person adds the alias after a check.
3. **The reader names the columns as the header writes them.** A caller of the table reader of
   `dim.tsv` finds the column `dimhash`.

## Options Considered

| Option | Checks the other 7 tables | Checks the columns of `dim.tsv` | Needs a list of headers in `sec` |
| ------ | ------------------------- | ------------------------------- | -------------------------------- |
| **A (chosen): metadata names and one alias** | Yes, by name | Yes, by name, with the alias | No |
| B: only the count of columns for `dim.tsv` | Yes, by name | Count only | No |
| C: headers compiled into `sec` | Against `sec`, not the metadata | Against `sec` | Yes, 8 lists |

- A keeps the check strict for each column, and holds one fact that the measurement shows.
- B misses a swap of the columns `segments` and `segt` of `dim.tsv`.
- C ignores the metadata member, so a new column in a file fails the file only through the
  compiled list.

## Consequences

- If the SEC renames `dimhash` to `dimh` in the header, every file fails. The fix removes the
  alias.
- The measurement covers 2 of 80 files. The alias is not measured on the other 78 files. On
  2026-10-28, the headers of all eight tables were equal in `2012q1`, `2024q1`, and `2026_09`.

## Open Questions

- Do other files differ in another column? The first full run of the transform answers it.

## Sources

- `2009q1_notes.zip`, downloaded on 2026-11-28 ([2009q1_notes.zip](https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2009q1_notes.zip), accessed 2026-11-28): the member `2009q1_notes-metadata.json` and the first line of each table.
- `2026_10_notes.zip`, read with HTTP range requests on 2026-11-28 ([2026_10_notes.zip](https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2026_10_notes.zip), accessed 2026-11-28): the ZIP index, the member `notes-metadata.json`, and the first line of each table.
