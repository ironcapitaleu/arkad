# Data Model & Storage Design — Index

Design artifacts from SPIKE
[STA-130](https://linear.app/state-machine/issue/STA-130/spike-design-hybrid-data-model-graph-knowledge-base-analytical-data)
(hybrid data model / storage abstraction). This README is the entry point — read in the order
below; each doc's header states its own status.

## Reading order & status

| # | Document | Status | What it is |
|---|----------|--------|------------|
| 1 | [`hybrid_data_model.md`](./hybrid_data_model.md) | **Authoritative** (findings complete) | The SPIKE findings: universal canonical core (LEI-keyed, SFAC-6-rooted) + regulator adapters, knowledge-graph & canonical-fact tiers, consistency/provenance model, storage-tech research, deployment options |
| 2 | [`storage_traits_design.md`](./storage_traits_design.md) | **Frozen (STA-145)** — the contract STA-139 scaffolds | The storage abstraction: composing `Repository` via associated types, `StorageError` currency, transaction ownership, crate topology, fakes. Signatures frozen; method-inventory questions resolved |
| 3 | [`load_superstate_design.md`](./load_superstate_design.md) | Exploratory, non-normative | The Load `SuperState` through the ports-and-adapters lens; for iteration before Load tickets are cut |
| 4 | [`design_patterns_primer.md`](./design_patterns_primer.md) | Background | Shared vocabulary: Repository, Ports & Adapters, Unit of Work, CQRS — each mapped onto arkad |
| 5 | [`design_patterns_demo.py`](./design_patterns_demo.py) | Background (runnable) | Dependency-free Python model of the same pattern combination: `python3 design_patterns_demo.py` |
| 6 | [`ontology_guardrails.md`](./ontology_guardrails.md) | Future reference, non-normative | Ontologies as guardrails / axiomatic enforcement on graph data: the arkad ↔ ontology mapping, reuse targets (FIBO, GLEIF L2, SKOS, Wikidata model), enforcement levels, and the adoption triggers for when to revisit |

## Decisions so far

- **Logical model settled** — universal core + regulator adapters; raw stores are the rebuildable
  system of record, graph + canonical facts are materializations (`hybrid_data_model.md` §2–§12).
- **Physical deployment deliberately deferred (2026-08-08)** — storage is fully abstracted behind
  the `storage` crate's ports; Option A / Option B (§14) is the framework for a *later*, trigger-
  gated decision, not a pending question.
- **Crate topology** — `xbrl` (domain vocabulary) ← `storage` (ports, no `sqlx`) ←
  `storage-postgres` (first concrete backend); only the composition root names a database.
- **Core = axiomatic identity; everything else is derived or a claim (2026-08-08 review
  iteration)** — the core splits into an axiomatic ring (`Company` + `Identifier`s, verifiable
  against registries) and a derived ring (canonical facts as provenance-attributed
  materializations); relationship edges are source-attributed claims that coexist rather than
  merge; multi-adapter facts coexist by `source_ref` with read-time selection; the fact table
  is the long-form time series. (`hybrid_data_model.md` §2.1, §4, §5, §6.1)

## What happens next

Tracked in Linear — the roadmap comment on STA-130 is the source of truth for the ticket series.
The **[DESIGN] ticket finalizing the `storage` crate (STA-145)** is done — the trait signatures in
`storage_traits_design.md` are frozen and the method-inventory questions resolved. Next up:
**STA-139** (scaffold the `storage` crate to those signatures), preceded by the small prerequisite
**`Lei` / `CompanyId` domain-concept** type (the identity newtype the signatures reference).
