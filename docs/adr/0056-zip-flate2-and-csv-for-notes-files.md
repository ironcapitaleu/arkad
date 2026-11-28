# ADR 0056: `zip`, `flate2`, and `csv` for the Notes Files

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-28 ([STA-566 comment `dfd39b27`](https://linear.app/state-machine/issue/STA-566#comment-dfd39b27), accessed 2026-11-28) |
| Date | 2026-11-28 |
| Deciders | The project owner |
| Ticket | [STA-566](https://linear.app/state-machine/issue/STA-566), accessed 2026-11-28 |

## Context

The Notes reader of the transform opens one stored Notes file and reads its tables. A Notes file
is a ZIP of eight tab-separated tables, a readme, and a metadata member. The reader runs two layers
of checks on it:

- **Layer 2, the container:** the ZIP signature, the ZIP index, the eight tables and one metadata
  member, deflate for every member, no path separator in a member name, and two size limits.
- **Layer 3, the table format:** UTF-8, fields split on tabs with quotes, a header equal to the
  columns of the metadata member, one field count per table, and the CRC-32 of each member.

The fields of a table follow the CSVW default dialect: a tab between fields, and `"` around a field
that holds a tab. Two rows of `dim.tsv` in `2024q1` hold such a field. A split on every tab reads
them with 4 fields instead of 3.

`sec` had no ZIP crate and no TSV crate. The choice binds the Notes reader of STA-566, the stream
split of STA-573, and the bucket split of STA-577. The bucket split reads `num.tsv` and `txt.tsv`
on two threads at the same time.

Terms:

- **Raw read:** a read of the zipped bytes of a member, as the ZIP stores them, without unzipping.
- **Size limit:** at most 16 GB unzipped per file, and at most 50 times the zipped size per member.

## Decision

1. **`zip` reads only the ZIP index.** `sec` depends on `zip` 9.4 with no default features, and
   with the feature `deflate-flate2` for the name of the deflate method. A raw read gives the
   offset and the zipped size of each member.
2. **`flate2` unzips each member.** The reader wraps the zipped bytes of one member in a
   `DeflateDecoder`. It counts the unzipped bytes and computes their CRC-32 with `flate2::Crc`. It
   stops one byte after the unzipped size of the ZIP index, so a member cannot unzip past its size
   limit.
3. **`csv` reads the fields.** The reader sets a tab as the delimiter, `"` as the quote, no header
   row, and a fixed field count. `csv` checks UTF-8 and the field count of each row.
4. **No type of `zip` or `csv` crosses the public interface of the reader.** The errors hold the
   text of a crate error, and a row is a type of `sec`.

## Options Considered

Measured on 2026-11-28 on `79f7276`, with `zip` 9.4.1, `flate2` 1.1.11, and `csv` 1.4.0.

| Option | New packages in `Cargo.lock` | Two tables at once | Own CRC-32 and size errors | Quote-aware reader |
| ------ | ---------------------------- | ------------------ | -------------------------- | ------------------ |
| **A (chosen): `zip` index, `flate2`, `csv`** | 2: `csv`, `csv-core` | Yes: each reader holds its own slice of the bytes | Yes | From `csv` |
| B: `zip` index, `flate2`, a reader written in `sec` | 0 | Yes | Yes | About 80 lines of code and their tests in `sec` |
| C: `zip` unzips, `csv` reads | 2 | No: `ZipArchive::by_index` borrows the archive as `&mut` | No: one `io::Error` with the text `Invalid checksum` | From `csv` |

- `zip` 9.4.1 was already in `Cargo.lock`, through the build dependency `utoipa-swagger-ui` of
  `api`. `flate2` was already a dependency of `sec`, behind the feature `test-fixtures`.
- `cargo tree -i openssl-sys --workspace` finds no package with option A. All three crates are
  pure Rust.
- A wins over B: `csv` is a reader with many users, and `sec` keeps no parser of its own to test.
- A wins over C: each table reader is independent of the archive, so two threads can read two
  tables. Each check fails with its own error, which names the table and both values.

## Consequences

- A table reader checks the CRC-32 and the size after its last row. A caller that stops early
  does not learn whether the member is intact.
- `csv` reads a quoted field with a newline, and a line that ends with CRLF, without an error. The
  Notes files hold neither, as measured on `2024q1`. Layer 3 does not check them.
- A move to another TSV crate changes only the table reader of `sec`, because no `csv` type is
  public.

## Open Questions

- If a Notes file uses another compression method than deflate, layer 2 fails the file. A new
  method reopens decision 2.

## Sources

- `zip` 9.4.1, `src/read/zip_archive.rs`: `by_index` and `by_index_raw` take `&mut self`.
- `zip` 9.4.1, `src/crc32.rs`: the CRC-32 error is `io::Error::new(io::ErrorKind::InvalidData,
  "Invalid checksum")`.
- `git diff 79f7276 -- Cargo.lock` on 2026-11-28: the packages `csv` 1.4.0 and `csv-core` 0.1.13
  are new.
- `cargo tree -i zip --workspace --target all -e all` on 2026-11-28: `zip` 9.4.1 comes through
  `utoipa-swagger-ui`.
