---
source: XBRL International specifications + FASB US GAAP 2026 Taxonomy XML files
last-verified: 2026-06-14
update-frequency: never (XBRL spec is stable; taxonomy XML updates annually but format stays the same)
---

# Linkbase XML Anatomy — Reading Guide

This reference teaches how to read the raw FASB taxonomy XML files stored in `../data/taxonomy/2026/` (relative to this references directory). The file format follows the XBRL 2.1 and Dimensions 1.0 specifications.

## File Naming Convention

`us-gaap-stm-{statement}-{variant}-{linkbase}-{year}.xml`

- **statement**: `sfp` (balance sheet), `soi` (income statement), `scf` (cash flow), `soc` (comprehensive income), `sheci` (stockholders' equity), `spc` (parenthetical)
- **variant**: `cls` (classified), `indir` (indirect method), `dir` (direct method), `dbo` (by order), etc.
- **linkbase**: `cal` (calculation), `pre` (presentation), `def` (definition)

## Common XML Structure

All linkbase files share this skeleton:

```xml
<link:linkbase xmlns:link='http://www.xbrl.org/2003/linkbase'
               xmlns:xlink='http://www.w3.org/1999/xlink'>
  <link:roleRef ... />           <!-- ELR (Extended Link Role) declaration -->
  <link:arcroleRef ... />        <!-- Arc role declaration (relationship type) -->
  <link:{type}Link xlink:role='...' xlink:type='extended'>
    <link:loc ... />             <!-- Locators: pointers to concepts -->
    <link:{type}Arc ... />       <!-- Arcs: relationships between locators -->
  </link:{type}Link>
</link:linkbase>
```

## Locators (`<link:loc>`)

A locator points to a concept in the taxonomy schema:

```xml
<link:loc xlink:href='../elts/us-gaap-2026.xsd#us-gaap_Assets'
          xlink:label='loc_Assets'
          xlink:type='locator' />
```

- **`xlink:href`** — URL reference to the concept definition. The fragment after `#` is the concept ID (format: `{namespace}_{ConceptName}`)
- **`xlink:label`** — Local identifier used by arcs to reference this locator. Convention: `loc_{ConceptName}`
- **`xlink:type='locator'`** — Always "locator"

To extract the concept name: strip the `loc_` prefix from the label, or parse the fragment from the href.

## Calculation Linkbase (`-cal-` files)

### Purpose

Defines arithmetic (summation) relationships: "parent = sum of children with weights". Used for Tier 3 derivation.

### Arc Role

```xml
<link:arcroleRef arcroleURI='https://xbrl.org/2023/arcrole/summation-item' ... />
```

The only arc role in calculation linkbases is `summation-item`: "the `to` concept is a summation contributor of the `from` concept".

### Calculation Arc Structure

```xml
<link:calculationArc order='10'
                     weight='1.0'
                     xlink:arcrole='https://xbrl.org/2023/arcrole/summation-item'
                     xlink:from='loc_Assets'
                     xlink:to='loc_AssetsCurrent'
                     xlink:type='arc' />
```

- **`xlink:from`** — Parent concept (the sum)
- **`xlink:to`** — Child concept (a contributor to the sum)
- **`weight`** — Multiplier: `1.0` means additive, `-1.0` means subtractive
- **`order`** — Display/processing order among siblings (lower = first)
- **`xlink:arcrole`** — Always `summation-item` for calculation

### How to Read

`from=Assets, to=AssetsCurrent, weight=1.0` means:
**Assets = ... + AssetsCurrent(×1.0) + ...**

Multiple arcs with the same `from` define all children:
```
Assets = AssetsCurrent(×1.0) + AssetsNoncurrent(×1.0)
```

Negative weight example (stockholders' equity):
```
StockholdersEquity = CommonStockValue(×1.0) + ... - TreasuryStockValue(×-1.0)
```

### ELR (Extended Link Role)

```xml
<link:calculationLink xlink:role='http://fasb.org/us-gaap/role/statement/StatementOfFinancialPositionClassified' ...>
```

The role URI scopes the relationships to a specific financial statement context. The same concept can have DIFFERENT parents in different ELRs. For example, `DepreciationAndAmortization` might be a child of `OperatingExpenses` in the income statement ELR but a child of `AdjustmentsToReconcile...` in the cash flow ELR.

**Always respect the ELR when traversing relationships.**

## Presentation Linkbase (`-pre-` files)

### Purpose

Defines parent-child display hierarchy. NOT arithmetic — just grouping and ordering for rendering financial statements. Useful for disambiguating where a concept belongs when it has no calculation relationship.

### Arc Role

```
http://www.xbrl.org/2003/arcrole/parent-child
```

### Presentation Arc Structure

```xml
<link:presentationArc order='10'
                      xlink:arcrole='http://www.xbrl.org/2003/arcrole/parent-child'
                      xlink:from='loc_StatementOfFinancialPositionAbstract'
                      xlink:to='loc_StatementTable'
                      xlink:type='arc' />
```

- **`xlink:from`** — Parent in display hierarchy
- **`xlink:to`** — Child in display hierarchy
- **`order`** — Rendering order among siblings
- **No `weight`** — presentation has no arithmetic semantics

### Key Difference from Calculation

- Presentation includes `Abstract` concepts (section headers that have no value)
- Presentation includes dimensional concepts (Table, Axis, Domain, Member)
- Presentation order defines how a statement looks when rendered
- A concept can appear in presentation without appearing in calculation (e.g., memo items, per-share data)

### When to Use Presentation

- To determine which financial statement a concept belongs on (income statement vs. cash flow)
- To understand grouping (is this concept under "Operating Expenses" or "Other Expenses"?)
- To identify concept hierarchy when calculation doesn't define the relationship

## Definition Linkbase (`-def-` files)

### Purpose

Defines dimensional relationships (XBRL Dimensions spec). Used to distinguish segment/axis breakdowns from consolidated totals. Critical for filtering SEC data.

### Arc Roles (multiple)

```xml
http://xbrl.org/int/dim/arcrole/all              — "this line item uses this hypercube"
http://xbrl.org/int/dim/arcrole/hypercube-dimension — "this hypercube has this axis"
http://xbrl.org/int/dim/arcrole/dimension-domain  — "this axis has this domain"
http://xbrl.org/int/dim/arcrole/dimension-default — "this axis defaults to this member"
http://xbrl.org/int/dim/arcrole/domain-member     — "this domain contains this member"
```

### Definition Arc Structure

```xml
<link:definitionArc order='1.0'
                    xbrldt:closed='true'
                    xbrldt:contextElement='segment'
                    xlink:arcrole='http://xbrl.org/int/dim/arcrole/all'
                    xlink:from='loc_StatementLineItems'
                    xlink:to='loc_StatementTable'
                    xlink:type='arc' />
```

- **`xbrldt:closed='true'`** — Only explicitly listed dimensions are valid (closed hypercube)
- **`xbrldt:contextElement='segment'`** — Dimension appears in the `segment` element of XBRL context

### Dimensional Hierarchy

```
LineItems (fact concepts)
  └── [all] → Table (hypercube)
                └── [hypercube-dimension] → Axis
                                            └── [dimension-domain] → Domain
                                                                     └── [domain-member] → Member
                                            └── [dimension-default] → DefaultMember
```

### When to Use Definition Linkbase

- To determine if a data point is a segment breakdown or consolidated total
- The SEC JSON API includes dimension info — cross-reference with definition linkbase to filter
- If a fact has no dimensions (or only the default member), it's the consolidated total
- If a fact has a specific member (e.g., `OperatingSegmentsMember`), it's a segment value

## Label Linkbase (`-lab-` files, in `elts/`)

### Purpose

Maps concept IDs to human-readable names. Located in `elts/us-gaap-lab-2026.xml` (13MB, covers all ~18,000 concepts).

### Structure

```xml
<link:labelLink ...>
  <link:loc xlink:href='us-gaap-2026.xsd#us-gaap_Assets' xlink:label='loc_Assets' ... />
  <link:labelArc xlink:from='loc_Assets' xlink:to='lab_Assets' ... />
  <link:label xlink:label='lab_Assets'
              xlink:role='http://www.xbrl.org/2003/role/label'
              xml:lang='en-US'>Assets</link:label>
</link:labelLink>
```

Label roles:
- `role/label` — Standard label (e.g., "Assets")
- `role/terseLabel` — Short form (e.g., "Assets")
- `role/verboseLabel` — Long form (e.g., "Assets, Total")
- `role/documentation` — Full definition text
- `role/totalLabel` — Used for totals (e.g., "Total Assets")
- `role/periodStartLabel` / `role/periodEndLabel` — For balance items shown at period boundaries

### When to Use Labels

- To display human-readable names for US GAAP concept IDs
- To resolve which concept a company might be using (search by label text)
- Documentation labels contain the full FASB Codification reference for the concept

## Practical Resolution Workflow

When resolving an unknown concept from SEC data:

1. **Check calculation linkbase** — Is it a child of any concept you already map? What's its weight?
2. **Check presentation linkbase** — Which statement section is it displayed under?
3. **Check label linkbase** — What's the human-readable name? Does it match a known canonical element?
4. **Check definition linkbase** — Is the data point dimensional? Is it a total or segment?

## Files in This Skill

```
../data/taxonomy/2026/
├── stm/
│   ├── us-gaap-stm-sfp-cls-cal-2026.xml    # Balance Sheet (classified) — calculation
│   ├── us-gaap-stm-sfp-cls-pre-2026.xml    # Balance Sheet (classified) — presentation
│   ├── us-gaap-stm-sfp-cls-def-2026.xml    # Balance Sheet (classified) — definition
│   ├── us-gaap-stm-soi-cal-2026.xml        # Income Statement — calculation
│   ├── us-gaap-stm-soi-pre-2026.xml        # Income Statement — presentation
│   ├── us-gaap-stm-soi-def-2026.xml        # Income Statement — definition
│   ├── us-gaap-stm-scf-indir-cal-2026.xml  # Cash Flow (indirect) — calculation
│   ├── us-gaap-stm-scf-indir-pre-2026.xml  # Cash Flow (indirect) — presentation
│   └── us-gaap-stm-scf-indir-def-2026.xml  # Cash Flow (indirect) — definition
└── elts/
    └── us-gaap-lab-2026.xml                 # Labels (all concepts, 13MB)
```
