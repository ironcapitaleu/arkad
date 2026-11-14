---
source: xbrl/DESIGN.md (section "3) Resolution", "Three Layers")
last-verified: 2026-11-09
update-frequency: on-design-change
---

# Resolution Tiers

The xbrl crate maps raw US GAAP concepts to CanonicalElements through four tiers of increasing complexity and decreasing confidence.

## Tier 1: Exact Match (Confidence::Exact)

The concept is the first concept of the alias list of a plug-in.

Example: `Assets` → CanonicalElement::Assets

## Tier 2: Synonym/Alias (Confidence::Synonym)

The concept is a later concept of the alias list, a part that the filing reports with no other
part (the part rule, `PartAliases`), or an ESEF anchor (planned `ifrs` plug-in).

Example: `RevenueFromContractWithCustomerExcludingAssessedTax` → CanonicalElement::Revenue (alias defined in REQUIRED_CONCEPTS for Revenue)

Alias priority: first match wins. Order aliases from most common/current to least.

Historical context: FASB taxonomy evolves concept names over time. Apple reported:

- Pre-2017: `SalesRevenueNet`
- 2017-2018: `Revenues`
- 2018+: `RevenueFromContractWithCustomerExcludingAssessedTax`

All three map to CanonicalElement::Revenue.

## Tier 3: Derived (Confidence::Derived)

An identity the core knows gives the value: an accounting identity or a period identity.

Examples:

- `Assets` absent, but `Liabilities` and `Equity` present. Derive: `Assets = Liabilities + Equity`
- `core/period/q4-from-fy@1`: Q4 = FY − 9-month year to date. It applies only to duration
  concepts in one currency, with no per-share unit.

## Tier 4: Computed (Confidence::Computed)

A sum along the calculation tree of a plug-in. The parent concept is absent, but ALL its children
are present.

Example: `Liabilities` absent, but `LiabilitiesCurrent` and `LiabilitiesNoncurrent` present.
Compute: `Liabilities = LiabilitiesCurrent + LiabilitiesNoncurrent`

The procedure is in `SKILL.md`, "Computation Rules (Tier 4)".

## Confidence Model

| Tier | Confidence | Mechanism |
| --- | --- | --- |
| 1 | Exact | The first concept of the alias list of a plug-in |
| 2 | Synonym | A later concept of the alias list, a part with no other part reported, or an ESEF anchor (planned `ifrs` plug-in) |
| 3 | Derived | An identity the core knows: an accounting identity or a period identity |
| 4 | Computed | A sum along the calculation tree of a plug-in |

The level of a derived or computed value is the weakest of its own level and the levels of its
inputs. Consumers can filter by level, for example a screener that takes only Exact and Synonym.
