# ADR 0039: Gzip Files for the Fixture Set of the `xbrl` Crate

| Field | Value |
| ----- | ----- |
| Status | Accepted by the owner on 2026-11-21 ([STA-539 comment `e10d9168`](https://linear.app/state-machine/issue/STA-539#comment-e10d9168)) |
| Date | 2026-11-21 |
| Deciders | The owner |
| Ticket | [STA-539](https://linear.app/state-machine/issue/STA-539) |

## Context

The acceptance tests of the `xbrl` crate need the Notes rows of real filings from 33 companies, and
the instance documents of 5 filings. The owner accepted the set of companies on 2026-11-16
(STA-539). No test downloads from the SEC, so the files live in the repository, in
`xbrl/src/tests/fixtures/data/`.

The set holds 102 filings: two 10-Ks per company, three 10-Qs for 10 companies, the Deckers 10-QT of
2014, and the 5 filings of the parser agreement set. Each filing has five Notes tables: `sub`,
`num`, `txt`, `dim`, and `tag`. The rows stay as the SEC wrote them.

STA-539 sets a limit: if the set adds more than 50 MB, the owner decides before the rest is cut.
The set as plain text is larger than the limit. The acceptance tests of STA-543 and every later test of the
set read the files in the format this ADR sets.

Terms: a *fixture file* is one file in the repository. Its *content* is the bytes the SEC wrote,
after decompression for a gzip file.

## Decision

1. **Each fixture file of the set is a gzip file.** A Notes table is `<table>.tsv.gz`, and an
   instance document is `<name>_htm.xml.gz`. The writer sets no file name and a modification time
   of 0 in the gzip header, so the same content always gives the same file.
2. **Tests read the files at run time.** `fixture_content` reads the file below
   `CARGO_MANIFEST_DIR` and decompresses it with `flate2`. The crate turns `flate2` on only with the
   feature `test-fixtures`. No test reaches the network.
3. **`manifest.tsv` lists every fixture file**, with the company, the CIK, the accession, the form,
   the fiscal year, the fiscal period, the source Notes file, and two SHA-256 digests. `sha256`
   covers the file as stored. `sha256_uncompressed` covers its content. A test recomputes both.
4. **The 10-Q `0001628280-26-068246` stays plain and compiled in.** Its files are 1.0 MB, and 9
   test files use it through `include_str!` constants.

## Options Considered

Measured on 2026-11-21 on the 102 filings, cut from 19 Notes files of the SEC.

| Option | Size in the working tree | Content as the SEC wrote it | New dependency |
| ------ | ------------------------ | --------------------------- | -------------- |
| **A: gzip per file (chosen)** | 13.5 MB (Notes 13.1 MB, instances 0.4 MB) | Yes, after decompression | `flate2`, optional |
| B: plain text, full set | 92.5 MB (Notes 83.9 MB, instances 8.6 MB) | Yes | None |
| C: plain text, no `txt` rows | about 70 MB, estimated from the share of `txt` in `num` and `txt` | No `txt` rows | None |
| D: plain text, fewer filings | under 50 MB | Yes | None |

- A stays under the limit with the full set. `flate2` uses the Rust backend `miniz_oxide`, links no
  C library, and is already in `Cargo.lock` through the `sec` crate.
- B is 42.5 MB larger than the limit. A git pack compresses the text, but every checkout holds 92.5 MB.
- C is still larger than the limit, and the parser sees no text facts.
- D drops filings that the owner accepted for the acceptance tests.

Run-time reads won over `include_bytes!`: one loader takes an accession, the test binary stays
small, and 102 filings need no constant each.

## Consequences

- A reviewer cannot read a fixture file in the diff. `zcat` shows its content, and the manifest
  names its source.
- A changed byte in a file or in its content fails the manifest test.
- A gzip file is a binary file to git. A change to one row replaces the whole file in the history.
  The set is cut once, so this cost stays small.
- The decision covers the files of this set. It does not cover a fixture outside
  `xbrl/src/tests/fixtures/data/`, or a fixture of another crate.

## Open Questions

- If the acceptance tests need `pre` or `cal` rows, the set grows. The same format holds, and the
  manifest gets one row per new file.
- If a later set is larger than 50 MB in gzip, the owner decides again, for example on Git LFS.

## Sources

- Sizes: `du -sb` on the cut files and the gzip files, 2026-11-21. The cut covered the Notes files
  `2013q2`, `2013q3`, `2013q4`, `2014q1`, `2014q2`, `2015q1`, `2024q2`, `2024_08`, `2024_10`, `2024_12`,
  `2025q1`, `2025q2`, `2025q3`, `2025_10`, `2025_11`, `2026_02`, `2026_03`, `2026_07`, and
  `2026_08`, at `https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/`.
- The owner's choice of option A: the answer to the size question in the session of STA-539,
  2026-11-21.
- `flate2` in the `sec` crate: `sec/Cargo.toml` on `main` at `33ad68d`.
