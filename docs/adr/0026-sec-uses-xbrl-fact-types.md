# ADR 0026: `sec` Uses the Fact Types of `xbrl` Directly

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms. Amendment 1: Proposed. The project lead confirms |
| Date | 2026-11-15 |
| Deciders | The project lead |
| Ticket | [STA-471](https://linear.app/state-machine/issue/STA-471/spike-plan-the-move-of-sec-financial-types-onto-xbrl) |

## Context

`sec/src/lib/shared/financial` holds a fact model of its own. `xbrl` holds a second one: the as-is
fact `AsIsFact`, the `Filing`, and the parser `xbrl::adapter::edgar::company_facts::parse`. The
state `ParseCompanyFacts` of `sec` parses the Company Facts JSON with its own code, so the JSON has
two parsers.

The two models differ in shape, not only in name:

- `sec` keeps a value as `i64`. `xbrl` keeps it as `BigDecimal`.
- `sec` has a closed `Unit` enum and a closed `Form` enum. `xbrl` has an open `Unit` and an open
  `Form`.
- `sec::shared::financial::period::Period` has `Instant { date }` and `Duration { start, end }`.
  `xbrl::core::period::Period` has `Instant { end }` and a `Duration` with a count of quarters.
- `sec::shared::financial::filing_source::FilingSource` holds the accession number, form, fiscal
  year, fiscal period, filing date, and period end. `xbrl::core::fact::FilingSource` holds the
  filing ID, filing date, and acceptance time. `xbrl::core::filing::Filing` holds the rest.
- `sec::shared::financial::concept_definition::ConceptDefinition` names a concept by text and an
  expected unit. `xbrl::us_gaap::mappings::ConceptDefinition` names it by `CanonicalElementKind`
  and an alias rule.

The choice binds [STA-472](https://linear.app/state-machine/issue/STA-472), which carries out the
move, and each later `sec` state that reads facts.

## Decision

1. **`sec` imports the fact types of `xbrl` at each call site, and deletes its own copies.** No
   re-export and no wrapper type sits between the two crates.
2. **`ParseCompanyFacts` delegates to `xbrl::adapter::edgar::company_facts::parse`.** It keeps its
   check of the required concepts, and takes the list from `xbrl::us_gaap::mappings`.
3. **`AccessionNumber` and the `Form` enum stay in `sec`, outside the fact model.** They move from
   `sec::shared::financial` to `sec::shared`, because the extraction states use them to pick and
   fetch filings.

## Amendment 1: A Required Concept Passes by an Alias or a Part

Date: 2026-11-16.

**Reason.** [STA-472](https://linear.app/state-machine/issue/STA-472) ran `parse` and the required
check on 11 real Company Facts bodies of `data.sec.gov`, fetched on 2026-11-16: CIKs 19617,
34088, 104169, 200406, 320193, 789019, 1018724, 1045810, 1067983, 1318605, and 1652044. `parse`
accepted all 11. A check by the `xbrl_keys` of `REQUIRED_CONCEPTS` alone passed 3 of them. Apple,
Microsoft, NVIDIA, and Alphabet report no `ProfitLoss` and no
`StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest`. They report the parts
`NetIncomeLoss` and `StockholdersEquity`, which `xbrl::us_gaap::mappings::PART_CONCEPTS` holds.

**Decision.** A required definition passes if a `us-gaap` fact has a local name in its
`xbrl_keys`, or in the `parts` of the `PART_CONCEPTS` entry of the same standard concept.

**Consequences.**

- 7 of the 11 bodies pass. JPMorgan Chase (19617) and Exxon Mobil (34088) report no operating
  income tag. Walmart (104169) and Amazon (1018724) report no `Liabilities`. The check of `sec` at
  `163c5c4` also rejects these 4 bodies, for the same missing tags.
- The check reads no `other_parts` and no `other_part_trees`. It asks whether a tag exists, not
  which value a period resolves to. `resolve_aliases` applies the full rule of a part.
- A standard concept that a company derives, such as liabilities from assets minus equity, is not
  covered. The ticket that adds derivations decides it.

## Options Considered

Measured on 2026-11-15 on `main` at `dee3756`.

| Option | Lines of `sec` types that stay | Fact models | Conversion of each fact |
| ------ | ------------------------------ | ----------- | ----------------------- |
| **A: direct replacement (chosen)** | 0 of 1,246 | One | None |
| B1: keep the `sec` types, with `From` from `xbrl` | 1,246 of 1,246, plus the `From` impls | Two | Can fail |
| B2: newtype wrappers around the `xbrl` types | 0 of 1,246, plus one newtype per type | One | None |
| C: re-exports | 0 of 1,246, plus one `pub use` per type | One | None |

"Lines of `sec` types" counts the lines before `#[cfg(test)]` in the 12 fact-model modules of
`sec/src/lib/shared/financial`. It leaves out `accession_number`, `form`, and `mod.rs`.

- A removes the second parser and the second model. Its cost is the import edits at each call
  site. Outside `shared/financial`, seven files name a fact-model type: six of the transform states
  and one transition. No binary and no integration test names one.
- B1 keeps both models, so they still drift apart. It keeps the `sec` shape, so each conversion
  from `xbrl` can fail: a `BigDecimal` value such as `1.23` does not fit `i64`, and an open form
  such as `20-F` has no variant in the `Form` enum.
- B2 wraps each `xbrl` type in a newtype, such as `struct Observation(AsIsFact)`. Its `From`
  cannot fail. It adds a second name for each `xbrl` type, as C does, and an accessor or `Deref`
  per type.
- C needs the same call-site edits as A, because the shapes differ. It then adds a second path,
  `sec::shared::financial::X`, for a type of `xbrl`. A reader must look up which crate owns it.

## Consequences

- `CompanyData` holds the `Cik`, the `xbrl` `EntityName`, and the `AsIsFact` and `Filing` values
  of the parse. It has no map keyed by `&'static ConceptDefinition`. A later state keys standardized
  facts by `CanonicalElementKind`.
- `CompanyData` stays the output of `ParseCompanyFacts` until `CreateFinancialStatements` emits
  `FactSet` values. That later step retires it, as `sec/design/data_model/hybrid_data_model.md`
  plans. This decision does not retire it.
- `ParseCompanyFacts` keeps every form, amendment, unit, and namespace. At `dee3756` it keeps only
  10-K and 10-Q filings with no amendment. The newest-filing dedup rule of `xbrl` picks one value per
  period.
- `ParseCompanyFacts` keeps values that are not integers, such as earnings per share. At
  `dee3756` it drops them.
- `ParseCompanyFacts` fails on a data point that `parse` rejects. At `dee3756` it drops that data
  point.
- `parse` accepts an `accn` of any shape with no whitespace. At `dee3756`, `ParseCompanyFacts`
  drops a data point whose `accn` is not an `AccessionNumber`. A later conversion from `FilingId`
  to `AccessionNumber` can fail.
- The required check reads no unit. At `dee3756`, a required concept with no fact in its expected
  unit counts as missing.
- The required list has 9 concepts in place of 11. Income tax becomes optional. `xbrl` maps no
  standard concept to shares outstanding. The `dei` fact stays an as-is fact.
- The `frame` field of a data point is not kept. No `sec` code reads it after the parse.
- The decision does not cover the Notes data set or instance documents. Their parsers are in
  `xbrl` already.

## Open Questions

- A real Company Facts body can hold a data point that `parse` rejects. STA-472 runs `parse` on
  real Company Facts bodies before it merges. If it finds such a data point, the rule of `parse`
  for it reopens, not this decision.
- If a statement needs shares outstanding, `xbrl` adds a standard concept for it. That ticket
  decides it.

## Sources

- `sec/src/lib/shared/financial/`: line counts by `awk '/#\[cfg\(test\)\]/{exit} {c++}'`,
  2026-11-15, `dee3756`.
- `sec/src/lib/implementations/states/transform/parse_company_facts/mod.rs`: `build_filing_source`,
  `parse_observation`, `resolve_concept`, `dee3756`.
- `xbrl/src/adapter/edgar/company_facts.rs`: `parse` and `Parser::add`, `dee3756`.
- `xbrl/src/us_gaap/mappings.rs`: 9 required and 43 optional definitions, `dee3756`.
- `sec/src/lib/shared/financial/concept_definition/constants.rs`: 11 required and 0 optional
  definitions, `dee3756`.
- Call sites: `grep -rln "financial::<module>"` over the workspace, 2026-11-15, `dee3756`.
