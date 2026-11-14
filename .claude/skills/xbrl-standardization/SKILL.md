---
name: xbrl-standardization
description: >
  Use when work on the xbrl crate turns filed facts into standardized facts: mapping US GAAP
  concepts to standard concepts, alias lists, resolution tiers and confidence levels, identity
  checks, the calculation tree, period identities such as Q4 from FY, dedup, the gap check, or
  adding a standard concept. Also use when the user asks "which concept is X", "add an alias", or
  "why is this value Derived". Not for parsers, I/O, or error types.
version: 0.3.4
---

# XBRL Standardization

## Purpose

This skill holds the rules by which the `xbrl` crate turns as-is facts into `StandardizedFact`
values. It covers resolution, the confidence levels, and the taxonomy data the rules read. The
accounting background, such as the accounting identities and the fiscal periods, is in the
`accounting` skill.

## Identity Checks

The standard concepts rest on the elements of FASB Concepts Statement 8, Chapter 4 (formerly
SFAC 6). Validation checks the identities in `xbrl/DESIGN.md`, "Layer 3", and the checks in
"Roll-Up Tree", "Checks the Tree Gives". The weakest confidence of the inputs of a failed check
tells what the failure means:

- All inputs `Exact`: the filing is inconsistent.
- At least one input `Synonym`, and none weaker: check the alias list first, then the filing.
- At least one input `Derived` or `Computed`: check the derivation first, then the filing.

The finding of a failed check carries the weakest confidence of its inputs.

## Resolution Procedure

When mapping a US GAAP concept to a CanonicalElement:

1. If the concept is the first concept of the alias list of a plug-in (`REQUIRED_CONCEPTS` or
   `OPTIONAL_CONCEPTS`), use it (Tier 1: Exact)
2. If it is a later concept of that alias list, a part that the filing reports with no other part
   (the part rule), or an ESEF anchor (planned `ifrs` plug-in), use it (Tier 2: Synonym)
3. If no alias resolves, apply an identity the core knows: an accounting identity or a period identity,
   such as Q4 = FY − 9-month year to date (Tier 3: Derived)
4. If still unresolved, sum the children of the concept along the calculation tree of a plug-in.
   The sum takes the arcs of one role (Tier 4: Computed)

Assign the `Confidence` level of the tier used. A derived or computed value takes the weakest level
of its own tier and its inputs.

## Adding New Concepts — Checklist

1. Identify the US GAAP concept name(s) from SEC filings or FASB taxonomy
2. Determine which CanonicalElement it maps to (see the elements in the `accounting` skill)
3. Research historical name variants (taxonomy versions change concept names over time)
4. Add to `REQUIRED_CONCEPTS` or `OPTIONAL_CONCEPTS` in `xbrl/src/us_gaap/mappings.rs`
5. Order aliases by priority (most common/current first)
6. Write tests verifying the mapping resolves correctly
7. Verify no accounting identity is violated by the new mapping

## Derivation Rules (Tier 3)

A Tier 3 value comes from an identity the core knows, not from the calculation tree:

1. An accounting identity, for example `Assets = Liabilities + Equity`
2. A period identity, for example `core/period/q4-from-fy@1`: Q4 = FY − 9-month year to date.
   It applies only to duration concepts in one currency, with no per-share unit
3. Store with `confidence: Derived`, or the weaker level of an input

## Computation Rules (Tier 4)

When a company does not report a parent concept, sum its children along the calculation tree:

1. Look up the children of the parent in the calculation arcs of one role
2. Check that the filing reports ALL children
3. Compute: `parent = Σ(child_i × weight_i)`, where the weight is +1 or −1
4. Store with `confidence: Computed`

Example: Amazon does not report `Liabilities` but reports `LiabilitiesCurrent` and
`LiabilitiesNoncurrent`. Compute: `Liabilities = LiabilitiesCurrent(+1) + LiabilitiesNoncurrent(+1)`

## Dedup and Gap Check

- `core/dedup/more-precise@1` picks one copy of a fact inside one filing. If the copies disagree,
  it reports a `Conflict`. The conflict blocks the candidate of that filing only if tiers 1 and 2
  would pick the conflicting concept. If the blocked candidate is the newest, the standard concept
  has no value for that period.
- `core/dedup/newest-filing@1` picks one value per standard concept and period across filings.
  The newest filing wins, amendments included. A restatement replaces the older value and sets
  `restated`.
- `core::chronology::check_chronology` checks each series over time. It reports `Gap`, `Overlap`,
  and `Restatement` findings and never fails.
- The rules are in `xbrl/DESIGN.md`, "4) Dedup and Chronology". The fiscal periods and forms they
  read are in the `accounting` skill.

## Critical Invariants

- Never map two different financial concepts to the same CanonicalElement for the same period
- Maintain traceability: every StandardizedFact must record its `rule` and its `inputs`

## Authoritative Sources

- **Accounting background:** the `accounting` skill
- **Calculation linkbase:** See `references/calculation-linkbase.md`
- **Current canonical elements:** See `references/canonical-elements.md`
- **Resolution tiers:** See `references/resolution-tiers.md`
- **FASB taxonomy (live):** `https://xbrl.fasb.org/us-gaap/{year}/`
  - Element definitions: `elts/us-gaap-{year}.xsd`
  - Calculation relationships: `stm/us-gaap-stm-*-cal-{year}.xml`
  - Labels: `elts/us-gaap-lab-{year}.xml`

## Raw Taxonomy Data

The authoritative FASB files live in the `xbrl` crate, in `xbrl/taxonomy/us-gaap/2026/` (relative
to the repository root). They are byte for byte the files of the FASB release zip.

- `stm/`: every statement group of the release, with its entry schema and its linkbases
  - `-cal-` files: arithmetic (summation) relationships with weights
  - `-pre-` files: display hierarchy and grouping
  - `-def-` files: dimensional relationships (segment vs. total)
- `elts/`: element-level data
  - `us-gaap-2026.xsd`: the element schema, with data type, balance, and period type
  - `us-gaap-lab-2026.xml`: human-readable labels for all concepts
  - `us-gaap-depcon-def-2026.xml` and `us-gaap-depcon-pre-2026.xml`: the deprecated concepts of
    the release, and the concept that replaces each one
- `MANIFEST`: the SHA-256 of each vendored file, sorted by path

The `srt` and `dei` releases that US GAAP 2026 imports live beside it, each with a `MANIFEST`:

- `xbrl/taxonomy/srt/2026/elts/`: `srt-2026.xsd` and `srt-lab-2026.xml`, from the FASB zip
  `https://xbrl.fasb.org/srt/2026/srt-2026.zip`
- `xbrl/taxonomy/dei/2026/`: `dei-2026.xsd` and `dei-2026_lab.xsd`, from
  `https://xbrl.sec.gov/dei/2026/`. The label linkbase sits in the `xs:appinfo` of a schema file

See `references/linkbase-anatomy.md` for how to read these XML files, and `xbrl/DESIGN.md`,
"Release in Use Today", for what the release holds.

## Staleness Check

Before using calculation linkbase relationships, check the `taxonomy-year` header in `references/calculation-linkbase.md`. If it is older than the current calendar year, prompt the user:

> "The FASB taxonomy reference is from {year}. A newer version may be available at xbrl.fasb.org/us-gaap/. Should I verify and update the references?"

For `references/canonical-elements.md`: if you notice CanonicalElement enum members in the source code that are not in the reference file, flag the mismatch to the user.

## Update Logic

### When to update taxonomy XML files (`xbrl/taxonomy/us-gaap/`)

FASB releases a new US GAAP taxonomy annually (typically Q1). Update when:

1. The current calendar year is newer than the `2026` in the directory path
2. The user explicitly asks to update to a newer taxonomy version
3. A concept is encountered that doesn't exist in the current taxonomy files

### How to update

1. Send a `HEAD` request for `https://xbrl.fasb.org/us-gaap/{new_year}/us-gaap-{new_year}.zip`.
   If it returns 200, the release exists. Never parse the HTML listing of the site.
2. Download the zip and unpack it in a new, empty directory.
3. Copy `elts/us-gaap-{year}.xsd`, `elts/us-gaap-lab-{year}.xml`, the two
   `elts/us-gaap-depcon-*-{year}.xml` files, and all of `stm/` into
   `xbrl/taxonomy/us-gaap/{new_year}/`. Never change a vendored file.
4. Write the `MANIFEST` in that folder. Run `find . -type f ! -name MANIFEST | sed 's|^\./||' |
   LC_ALL=C sort | xargs sha256sum > MANIFEST` inside the folder.
5. Find the `srt` and `dei` releases that the new release imports: search its files for
   `xbrl.fasb.org/srt/` and `xbrl.sec.gov/dei/`. Vendor their schema and label linkbase into
   `xbrl/taxonomy/srt/{year}/` and `xbrl/taxonomy/dei/{year}/`, with a `MANIFEST` each. In
   `xbrl/src/us_gaap/generator/mod.rs`, update `ELEMENT_SCHEMAS` and `CHECKSUM_FOLDERS`. In
   `xbrl/tests/us_gaap_manifest.rs`, update `RELEASE_IN_USE_FOLDERS`.
6. Update `references/calculation-linkbase.md` header: set `taxonomy-year` to the new year
7. Verify no existing concept mappings broke (concept names may have changed between versions)
8. Prompt user to remove old taxonomy directory if no longer needed

### What does NOT need updating

- `references/linkbase-anatomy.md` — XBRL spec is stable; file format does not change between taxonomy years
- `references/resolution-tiers.md` — only update if the crate's resolution architecture changes
