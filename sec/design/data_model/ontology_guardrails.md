# Ontologies as Guardrails — Axiomatic Enforcement on Graph Data (Future Reference)

> **Status:** future-direction reference, **non-normative**. Nothing here is scheduled work.
> Purpose: capture the ontology ↔ arkad mapping once, so the topic never has to be
> re-derived from scratch. Revisit when one of the **adoption triggers** (§6) fires.
>
> Origin: design discussion 2026-08-08, prompted by *"Why Agentic Systems Need Ontologies"*
> (Frank Coyle, UC Berkeley — AI Engineer World's Fair 2026, Track 5 "Graphs"):
> <https://www.youtube.com/watch?v=Sir59K8ZDPU>. Read alongside `hybrid_data_model.md`
> (§2.1 two rings, §3.2 invariants, §5 knowledge graph, §5.2 claims).

## 1. The goal, in one paragraph

If the knowledge-graph tier is later materialized as an actual graph, we may want an
**ontology layer on top of it: a machine-enforced schema of entity types, relation types,
and axioms** — so that graph writes and agentic consumers are constrained by *rules a
machine enforces*, not prose a reviewer hopes is followed. The design below shows that
arkad's data model is already ontology-shaped, what standard vocabularies to align with,
what an enforcement layer would concretely look like, and what we deliberately do **not**
adopt today.

## 2. arkad is already an ontology in disguise — the mapping

The current data model (post `ee3b00d` review rework) maps almost 1:1 onto ontology-world
concepts. This table is the core of the reference — it means "adding an ontology" later is a
*formalization* of what exists, not a redesign:

| arkad concept | Ontology equivalent | Notes |
| --- | --- | --- |
| `CanonicalElement` (SFAC-6-rooted vocabulary, 13 L1 roots + L2) | Concept scheme / class taxonomy (SKOS-shaped) | Our own curated vocabulary — axiomatic by construction |
| Axiomatic core ring: `Company` + `Identifier` (§2.1, §5.1) | Entities with registry-grounded identity (IRIs ↔ `CompanyId`) | |
| CIK ↔ LEI mapping; relink-without-rewrite (§4) | **`owl:sameAs`** identity reconciliation | Same entity known under multiple identifier schemes |
| Claim envelope on relationship edges — `source`, `as_of`, `observed_at`, `verifiability`; conflicts coexist; read-time selection (§5.2) | **Wikidata statement model**: references (source), qualifiers (point-in-time), ranks (preferred/normal/deprecated = read-time policy) | Independently reinvented; Wikidata is the prior art to consult when finalizing the claim layer |
| SFAC-6 `Invariant`s (Assets = Liabilities + Equity, …) + per-element DQ table (§5.2) | **OWL axioms / SHACL shapes** — machine-enforced constraints | "A sentence in a spec is a hope; an axiom is a rule a machine enforces" |
| Resolution engine deriving facts via calc-linkbase (`Derived`/`Computed` + `resolution_path`) | **RDFS-style inference** — the reasoner derives, unasked; `resolution_path` is the derivation trace | |
| Restatement invariant; exactly-one-primary-id | `owl:FunctionalProperty`-class constraints ("at most one") | |
| Regulator adapters resolving native tags → canonical vocabulary | Ontology alignment / mapping between vocabularies | US-GAAP and IFRS as *dialects* of one upper vocabulary |

**RDFS infers, OWL constrains** — the talk's split maps cleanly onto ours: the *derived
ring* (§2.1) is the inference side; the *DQ checks + invariants* are the constraint side.

## 3. What axiomatic enforcement on graph data would look like here

Three levels, in adoption order. Level 1 exists; levels 2–3 are the future option this doc
preserves:

1. **Today — axioms in Rust (already enforced).** The type system + `xbrl::Invariant` +
   the compiler-enforced crate boundaries *are* our axiom engine. A Rust newtype rejecting a
   malformed LEI is a stronger guarantee than a SHACL shape. Nothing to add.
2. **When the graph tier materializes — a declarative constraint layer over the graph.**
   The §5.2 per-element DQ table becomes an explicit, versioned **graph schema + constraint
   set** (node types, edge types, cardinalities, value ranges, conflict detectors) evaluated
   against the graph store — SHACL-style in spirit, whatever the engine (SQL CHECK +
   validation queries on Postgres; Cypher constraints on a graph DB; SHACL proper on an RDF
   store). Key property: **constraints live as data/config reviewed like code, not as prose
   in a doc.** This is the "axiomatic enforcement on top of graph data" target.
3. **If interop/agents demand it — an RDF/OWL projection as a read-side view.** Because raw
   is the replayable SoT and storage sits behind the `storage` ports, an RDF/JSON-LD export
   is *just another materialization* (one more impl behind the traits, §14.E). An OWL/SHACL
   reasoner can then run over that projection as an **offline auditor** (batch DQ audits,
   agent-facing guardrails) without ever becoming the system of record.

**Agentic relevance (the talk's thesis):** when an LLM/agent layer sits on top (screener
assistant, Q&A), the typed graph + claim envelope + invariants are precisely the guardrails
that prevent hallucinated joins and made-up enum values. Design intent: **agents query
through the ontology-shaped surface; they never get raw free-text.**

## 4. Reuse targets — "take advantage of what already exists"

Vocabulary/design-level borrowings; all zero runtime cost:

| Source | What to take | Where it lands |
| --- | --- | --- |
| **FIBO** (EDM Council, finance ontology) | Term alignment for legal entities, **ownership vs control** distinctions on relationship edges | Claim-edge naming/semantics (`SUBSIDIARY_OF`, `OWNS_STAKE_IN`) when the claim layer is finalized |
| **GLEIF LEI Level 2** ("who owns whom" relationship records; GLEIF publishes an RDF ontology) | A **direct data source** for ownership claims at the `Verified` tier — from the registry we already use for identity (Level 1) | Future ownership-claims adapter; roadmap candidate |
| **SKOS** | Concept-scheme + cross-scheme mapping model (`exactMatch`/`closeMatch`) | Industry/Sector claim layer (GICS vs SIC vs NACE mappings, §5.1) |
| **Wikidata statement model** | References/qualifiers/ranks pattern | Validation + refinement of the §5.2 claim envelope |
| **schema.org** (`Organization`) | Lightweight interchange vocabulary | Only if/when a public JSON-LD export exists (level 3) |

## 5. Explicit non-goals (today)

- **No triple store / RDF database as a tier.** The physical-storage decision is deferred
  behind the `storage` ports (`hybrid_data_model.md` §14, decision 2026-08-08); adopting an
  RDF store would re-open it. Triple-store + reasoner Rust support is thinner than the graph
  DBs already ruled out for day one (§13).
- **No OWL reasoner in the write path.** Enforcement today is Rust; a reasoner, if ever, is
  an offline auditor over a projection (level 3), never the ingestion gate.
- **No wholesale FIBO adoption.** Align names/semantics where they match; FIBO's full class
  hierarchy is far heavier than the platform needs.

## 6. Adoption triggers — when to reread this doc

1. **The graph tier is being materialized** (post-§14 trigger) → implement level 2:
   promote the §5.2 DQ table into a versioned, declarative constraint set over the graph.
2. **The claim layer is being finalized** (ownership/industry sources onboarded) → apply the
   §4 reuse targets: FIBO term alignment, GLEIF L2 as a `Verified` claims source, SKOS for
   scheme mappings, Wikidata's model as the envelope benchmark.
3. **An agent/LLM layer is being built on top** → design its query surface against the
   ontology shape (typed entities, claims with sources, invariant-checked facts); consider
   the level-3 projection if the agent framework consumes RDF/JSON-LD natively.
4. **An external interop requirement appears** (partner wants machine-readable KG) →
   level-3 JSON-LD/RDF export as a read-side materialization behind the storage ports.
