---
source: xbrl/DESIGN.md (Resolution Layer section)
last-verified: 2026-06-14
update-frequency: on-design-change
---

# Resolution Tiers

The xbrl crate maps raw US GAAP concepts to CanonicalElements through four tiers of increasing complexity and decreasing confidence.

## Tier 1: Exact Match (Confidence::Exact)

The raw concept key matches a known concept definition's primary key exactly.

Example: `Assets` → CanonicalElement::Assets

## Tier 2: Synonym/Alias (Confidence::Synonym)

The raw concept key matches one of the known aliases for a concept.

Example: `RevenueFromContractWithCustomerExcludingAssessedTax` → CanonicalElement::Revenue (alias defined in REQUIRED_CONCEPTS for Revenue)

Alias priority: first match wins. Order aliases from most common/current to least.

Historical context: FASB taxonomy evolves concept names over time. Apple reported:

- Pre-2017: `SalesRevenueNet`
- 2017-2018: `Revenues`
- 2018+: `RevenueFromContractWithCustomerExcludingAssessedTax`

All three map to CanonicalElement::Revenue.

## Tier 3: Derived (Confidence::Derived)

The parent concept is absent but ALL required children are present. Compute from calculation linkbase relationships.

Example: `Liabilities` absent, but `LiabilitiesCurrent` and `LiabilitiesNoncurrent` present. Derive: `Liabilities = LiabilitiesCurrent + LiabilitiesNoncurrent`

Rules:

- ALL children must be present (partial sums are unreliable)
- Use weights from calculation linkbase (+1 or -1)
- Confidence is Derived, not Exact

## Tier 4: Computed (Confidence::Computed)

Full FASB linkbase graph traversal. Multi-hop relationships, indirect derivation.

Example: Deriving `OperatingIncome` when neither it nor `GrossProfit` is reported, but `Revenue`, `CostOfRevenue`, and all `OperatingExpenses` components are present.

Status: Not yet implemented. Requires calculation linkbase parser in `us_gaap/taxonomy.rs`.

## Confidence Model

| Tier | Confidence | Meaning |
| --- | --- | --- |
| 1 | Exact | Direct key match — highest trust |
| 2 | Synonym | Known alias — high trust |
| 3 | Derived | Computed from direct children — medium trust |
| 4 | Computed | Multi-hop graph traversal — lower trust |

Downstream consumers can filter by confidence level (e.g., screener only uses Exact+Synonym).
