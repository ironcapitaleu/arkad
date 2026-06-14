---
name: xbrl-accounting
description: >
  Use when working on the xbrl crate, concept resolution, normalization,
  SFAC 6 invariants, US GAAP taxonomy concepts, calculation linkbase,
  or expanding alias/concept coverage.
version: 0.1.0
---

# XBRL Accounting Domain Knowledge

## Purpose

This skill provides accounting domain knowledge for the xbrl crate's financial concept normalization. It ensures correct application of SFAC 6 identities, guides the resolution procedure, and prevents hallucination on accounting standards.

## SFAC 6 Foundation

The crate's canonical model is built on SFAC 6 (Statement of Financial Accounting Concepts No. 6, 1985). Five invariants must always hold:

1. **Balance Sheet Identity:** Assets = Liabilities + Equity
2. **Income Statement Identity:** Net Income = Revenue - Expenses + Gains - Losses
3. **Comprehensive Income:** Comprehensive Income = Net Income + OCI
4. **Changes in Equity:** ΔEquity = Comprehensive Income + Investments by Owners - Distributions to Owners
5. **Cash Flow Reconciliation:** Operating CF + Investing CF + Financing CF ≈ ΔCash

These are the validation checks the system enforces. Violation of identity #1 is always a bug in the system, never valid company data.

## Resolution Procedure

When mapping a US GAAP concept to a CanonicalElement:

1. Check if the concept key matches exactly in `REQUIRED_CONCEPTS` or `OPTIONAL_CONCEPTS` (Tier 1: Exact)
2. Check if it matches any known alias in a ConceptDefinition's alias list (Tier 2: Synonym)
3. If no direct mapping, check if the concept can be derived from children via calculation linkbase (Tier 3: Derived)
4. If still unresolved, traverse the full FASB linkbase graph for indirect relationships (Tier 4: Computed)

Always assign the appropriate `Confidence` level matching the tier used.

## Adding New Concepts — Checklist

1. Identify the US GAAP concept name(s) from SEC filings or FASB taxonomy
2. Determine which CanonicalElement it maps to (consult SFAC 6 element definitions)
3. Research historical name variants (taxonomy versions change concept names over time)
4. Add to `REQUIRED_CONCEPTS` or `OPTIONAL_CONCEPTS` in `xbrl/src/us_gaap/mappings.rs`
5. Order aliases by priority (most common/current first)
6. Write tests verifying the mapping resolves correctly
7. Verify no SFAC 6 identity is violated by the new mapping

## Derivation Rules (Tier 3)

When a company doesn't report a parent concept, derive it from children:

1. Look up the parent's children in the calculation linkbase
2. Check if ALL required children are present in the filing
3. Compute: `parent = Σ(child_i × weight_i)` where weight is +1 or -1
4. Store with `confidence: Derived`

Example: Amazon doesn't report `Liabilities` but reports `LiabilitiesCurrent` + `LiabilitiesNoncurrent`. Derive: `Liabilities = LiabilitiesCurrent(+1) + LiabilitiesNoncurrent(+1)`

## Critical Invariants

- Never map two different financial concepts to the same CanonicalElement for the same period
- Preserve sign conventions: expenses are positive values that reduce net income
- Maintain traceability: every ResolvedFact must record its resolution_path
- Balance sheet items are Instant (point-in-time); income/cash flow items are Duration
- Amendment filings supersede originals (same accession = latest wins)

## Authoritative Sources

- **SFAC 6 definitions:** See `references/sfac6-elements.md`
- **Calculation linkbase:** See `references/calculation-linkbase.md`
- **Current canonical elements:** See `references/canonical-elements.md`
- **Resolution tiers:** See `references/resolution-tiers.md`
- **FASB taxonomy (live):** `https://xbrl.fasb.org/us-gaap/{year}/`
  - Element definitions: `elts/us-gaap-{year}.xsd`
  - Calculation relationships: `stm/us-gaap-stm-*-cal-{year}.xml`
  - Labels: `elts/us-gaap-lab-{year}.xml`

## Staleness Check

Before using calculation linkbase relationships, check the `taxonomy-year` header in `references/calculation-linkbase.md`. If it is older than the current calendar year, prompt the user:

> "The FASB taxonomy reference is from {year}. A newer version may be available at xbrl.fasb.org/us-gaap/. Should I verify and update the references?"

For `references/canonical-elements.md`: if you notice CanonicalElement enum members in the source code that are not in the reference file, flag the mismatch to the user.
