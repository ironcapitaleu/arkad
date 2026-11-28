# ADR 0058: A Generated Table of the Removed US GAAP Elements

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project owner on 2026-11-28 ([STA-619 comment `b118ce56`](https://linear.app/state-machine/issue/STA-619#comment-b118ce56), accessed 2026-11-28) |
| Date | 2026-11-28 |
| Deciders | The project owner |
| Ticket | [STA-619](https://linear.app/state-machine/issue/STA-619), accessed 2026-11-28 |

## Context

The plug-in `us_gaap` takes its reference data from the US GAAP release 2026. The repository
vendors the files of that release with their bytes, and a `MANIFEST` names the SHA-256 of each
file. A test runs the generator on the vendored files and compares its output with the checked-in
tables.

The release 2026 does not define the elements that earlier releases removed, such as
`SalesRevenueNet`. Old filings use them. On the fixture set, 8,758 of 81,829 US GAAP facts of
filings before 2022 (10.7%) use a removed element. Such a fact still resolves, because the alias
and part rules match a concept by name. It lacks only its reference data: label, data type,
balance, and period type. The silver dataset `taxonomy_elements` needs that data for each concept
([STA-484](https://linear.app/state-machine/issue/STA-484), accessed 2026-11-28).

In this document, a **removed element** is a concept of a US GAAP schema of 2011 to 2025 that the
schema of 2026 does not define. A **source file** is the element schema or the label linkbase of
the last release that defines a removed element.

- The schemas of 2011 to 2025 hold 6,628 removed elements. The last releases that define them are
  the 14 releases 2012 to 2025. So the source files are 14 schemas and 14 label linkbases.
- The FASB does not change a published release.

The decision binds the plug-in `us_gaap`, its generator, STA-484, and each later release that
removes elements.

## Decision

1. **A generator reads the 28 source files once and writes one table of the removed elements.**
   The repository keeps the generated table, not the source files. The table holds one row per
   removed element: name, data type, balance, period type, standard label, the last release that
   defines it, and the release that its label names as the deprecation date.
2. **A `MANIFEST` names the URL and the SHA-256 of each source file.** The generator refuses a file
   whose SHA-256 differs from the `MANIFEST`. A developer downloads the files from the URLs before
   a run.
3. **The trait `RemovedElements` gives the table to the core, next to `Taxonomy`.**
   `Taxonomy::element()` and `Taxonomy::elements()` keep describing the release in use.

## Options Considered

Measured on 2026-11-28 with the files of `xbrl.fasb.org` and the fixture set of `xbrl` on `main`
at `083de59`.

| Option | Size per checkout | Offline generator test | New infrastructure |
| ------ | ----------------- | ---------------------- | ------------------ |
| **A. Generated table, no source files (chosen)** | 1.8 MB of Rust source | No | None |
| B. Vendor the 28 source files in Git | 263 MB, about 15 MB packed | Yes | None |
| C. Vendor the source files through Git LFS or GCS | Small | No, it needs the network | LFS or GCS access for CI and each session |
| D. Vendor the 14 schemas only | 64 MB | Yes | None |
| E. One plug-in release per year | About 30 MB for each of 15 releases | Yes | 15 generated tables |
| F. No reference data for removed elements | None | Not applicable | None |

- A adds no infrastructure and keeps each checkout small. The source files never change, so an
  offline test that recomputes the table guards only against a change to the generator. The
  generator test of the release 2026 covers the readers of schemas and labels that both use.
- B keeps the bytes of the publisher. Each checkout grows by 263 MB, nine times the 30 MB of the
  vendored release 2026. Cloud sessions have a fixed disk allowance.
- C keeps the bytes outside Git, but CI and every session then need access to the store.
- D has no labels. The deprecation date lives in the label, so the release that removed an element
  becomes the first release without it.
- E gives each filing the calculation tree of its own release. That gain is in tier 4 only, which
  no rule uses yet.
- F leaves 10.7% of the US GAAP facts before 2022 without reference data.

## Consequences

- The plug-in compiles in 6,628 more rows. The generated file is 1.8 MB of Rust source.
- A change to the generator of the removed elements needs a run with the downloaded source files.
  No test in CI runs it.
- The repository departs from the rule that it keeps the bytes of each publisher file it reads.
  The `MANIFEST` keeps the SHA-256 of each source file, so a later run can check the bytes.
- STA-484 writes the removed elements into `taxonomy_elements`, with `retired_in` from the
  deprecation date, or from the first release without the element if the label names no date.
- If a release of 2027 removes elements of 2026, the element schema and the label linkbase of 2026
  join the `MANIFEST` as source files.
- Not covered: the calculation arcs of earlier releases. A one-off read of a calculation linkbase
  answers a question about an old tree.
- Not covered: the removed elements of `srt` and `dei`. No fixture fact of these families uses a
  removed element.

## Open Questions

- If the FASB moves or deletes a source file, the generator cannot run again. Then option B or C
  becomes the fallback.

## Sources

- [Linear document "US GAAP Elements Removed Before 2026: Findings"](https://linear.app/state-machine/document/us-gaap-elements-removed-before-2026-findings-dc4953510c40),
  accessed 2026-11-28: the share of facts, the sizes, and the options.
- The element schemas and label linkbases of the releases 2011 to 2025 on `xbrl.fasb.org`, for
  example ([us-gaap-2020-01-31.xsd](https://xbrl.fasb.org/us-gaap/2020/elts/us-gaap-2020-01-31.xsd),
  accessed 2026-11-28): the 6,628 removed elements and the 28 source files.
- `xbrl/taxonomy/us-gaap/2026/elts/us-gaap-2026.xsd` at `083de59`: the concepts of the release
  2026.
