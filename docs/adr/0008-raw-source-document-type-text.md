# ADR 0008: Each Raw Source Holds Its Document Type as Text

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms. |
| Date | 2026-11-14 |
| Deciders | The project lead |
| Ticket | [STA-350](https://linear.app/state-machine/issue/STA-350) researched the options. Linear document "STA-350 Findings: How Raw Sources Are Named" holds the findings. |

## Context

`RawDocument<S>` in `sec` implements the `Document` trait of the filesystem adapter in `storage`.
`Document` requires `type DocumentType: fmt::Display` and `const DOCUMENT_TYPE`. The store writes
the `Display` text of `DOCUMENT_TYPE` into the metadata of each document as `document_type`.

Each raw source is a marker type that implements `RawSource`, such as `RawNotesArchive`. Before this
decision, each source also had a variant of the enum `SecDocumentType`, and each marker named its
variant. So each source had two names that had to agree, and a hand-written `Display` match gave
the text. Eight sources exist: `RawCompanyFacts`, `RawDailyIndex`, `RawFilingIndex`,
`RawFullIndex`, `RawNotesArchive`, `RawSubmissions`, `RawXbrlInstance`, and `RawXbrlLabels`.

The store compares the text, not the Rust type. Every writer of `sec` uses the one root that
`ARKAD_STORAGE_ROOT` names. If two sources hold one text, a listing of one returns the files of
the other, and a read accepts a file of the other. No check across sources existed. Each marker had
one test that its constant named its own variant.

The choice binds every later raw source. A person questioned the pair during STA-338
(ironcapitaleu/arkad#278).

Terms:

- **Stored text**: the `document_type` value in the metadata of a stored document.

## Decision

1. **Each marker holds its stored text as `const DOCUMENT_TYPE: &'static str`.** `SecDocumentType`
   does not exist. `RawDocument<S>` sets `type DocumentType = &'static str`.
2. **One test in `sec/src/lib/shared/raw_source/implementations/` fails if two markers hold one
   text.** A new marker adds its constant to the list of that test.
3. **The `Document` trait in `storage` does not change.** `&'static str` implements `Display`.

## Options Considered

Read on 2026-11-14 on `main` at `83f585d`.

| Option | Edits per new source | Guard against a shared text | `storage` trait changed |
| ------ | -------------------- | --------------------------- | ----------------------- |
| **A (chosen):** text constant on the marker, and a uniqueness test | Marker, test list | Test | No |
| B: keep the enum, and a uniqueness test | Marker, variant, `Display` arm, test list | Test | No |
| C: no change | Marker, variant, `Display` arm | Review | No |
| D: `Document` takes `const DOCUMENT_TYPE: &'static str` and no associated type | Marker, test list | Test | Yes |

- A gives one name per source in code and changes no trait.
- B keeps two names that must agree. Its test must compare the texts, not the variants, because
  two variants can share one `Display` text.
- C has no guard across sources.
- D modifies `Document` and all 17 of its implementors, including 12 doctests. A works
  without it, so the Open-Closed Principle in `AGENTS.md` rules it out.

No option stops a shared text at compile time. The test sees only the markers that it lists,
because Rust cannot list every implementor of a trait.

## Consequences

- A new raw source is one marker file and one entry in the test list.
- The stored texts did not change, so every stored document still reads. Each marker keeps a test
  that pins its text.
- The document type of a raw source is no longer a value of a closed enum. A `match` over all
  sources needs a list of its own.
- A marker that the test list omits is not checked.
- The rule that a text is unique across every crate that stores documents has no test. Only `sec`
  implements `Document` outside `storage`.

## Open Questions

- Whether a macro generates the markers and the test list from one list, so that the test cannot
  miss a marker. A further raw source, or a missed marker, reopens this question. The project lead
  decides.

## Sources

- `storage/src/implementations/repository/filesystem/traits/document/mod.rs` on `main` at
  `83f585d`, read on 2026-11-14: `type DocumentType: fmt::Display`.
- The four readers of `D::DOCUMENT_TYPE.to_string()` under
  `storage/src/implementations/repository/filesystem/`: `shared/document_metadata/builder`,
  `implementations/write_repository`, `implementations/read_repository`, and
  `implementations/list_repository`, on `main` at `83f585d`, read on 2026-11-14.
- `sec/src/bin/shared/storage_root.rs` on `main` at `83f585d`: every writer uses one root.
- The count of 17 `Document` implementors: `git grep 'impl Document for'` on `main` at `83f585d`,
  run on 2026-11-14. It finds 12 doctests, 3 `SampleDocument` fixtures, `BareDocument`, and
  `RawDocument<S>`, of which 16 are in `storage`.
