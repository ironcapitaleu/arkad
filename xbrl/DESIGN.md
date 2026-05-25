# XBRL Crate Design Document

## General Thoughts

Create a new crate `xbrl`.

Start with data for US-GAAP first.

**Parsing**: extracting the data from a JSON response body from the SEC API.

**Validation**: look for three things in financial statements:
- Consistency
- Completeness
- Precision

## Draft Crate Structure

```
xbrl/
  core/
    elements.rs       <- CanonicalElement enum + SFAC 6 invariants
    resolved_fact.rs  <- ResolvedFact (value + confidence + provenance + resolution path)
    fact_set.rs       <- coherent collection for one entity + period
    graph.rs          <- resolution + validation engine
    period.rs         <- Instant | Duration distinction
    confidence.rs     <- Exact | Synonym | Derived | Computed
    observation.rs    <- RawObservation type
    provenance.rs     <- Provenance type (filing metadata)
  us_gaap/
    mappings.rs       <- concept name -> canonical element (Tier 1 & 2 synonyms)
    taxonomy.rs       <- FASB linkbase relationships (the calculation tree)
  sec_api/
    company_facts.rs  <- JSON deserializer for /companyfacts/ endpoint
    company_concept.rs <- JSON deserializer for /companyconcept/ endpoint
    frames.rs         <- JSON deserializer for /frames/ endpoint
```

**Crate boundary**: `xbrl::sec_api` takes `&str` or `&serde_json::Value` (raw JSON body), NOT a `SecResponse`. The `sec` crate owns HTTP, rate limiting, and response handling. `xbrl` has zero network dependencies.

---

## 0) Data Model

Define the target model inside `xbrl::core`, outside any taxonomy-specific module, to contain core financial company data required for all companies.

Start with SFAC 6.

SFAC 6 is the authoritative definition of what financial data exists and how it relates. It describes 10 elements as well as the invariants that connect them. Unchanged since 1985.

It gives you:
- **What to extract** — the 10 elements are the canonical set
- **How to validate** — the invariants are the correctness checks
- **How to derive** — if one element is missing, solve for it from the identity (e.g., missing Equity = Assets - Liabilities)

### 0.1) Balance Sheet

3 roll-up items:
- Assets
- Liabilities
- Equity

Invariant: `Assets = Liabilities + Equity`

### 0.2) Cash Flow Statement

3 roll-up items:
- Operating Cash Flow
- Cash Flow from Investing Activities
- Financing Cash Flow

### 0.3) Income Statement

Roll-up items:
- Revenue
- Expenses
- Gains
- Losses
- Net Income (= Revenue - Expenses + Gains - Losses)
- Comprehensive Income (= Net Income + OCI items)

### 0.4) Changes in Equity

Links the other statements. Explains additional investment / payout of company substance.

`ΔEquity = Comprehensive Income + Investments by Owners - Distributions to Owners`

### CanonicalElement Enum

> **[DECISION NEEDED]** Enumerate the full set before implementation. Draft below.

```rust
enum CanonicalElement {
    // --- SFAC 6 Level 1 (root invariants) ---
    Assets, Liabilities, Equity,
    Revenue, Expenses, Gains, Losses, NetIncome,
    ComprehensiveIncome,
    OperatingCashFlow, InvestingCashFlow, FinancingCashFlow,

    // --- Level 2: Balance Sheet sub-elements ---
    CurrentAssets, NonCurrentAssets,
    CashAndEquivalents, ShortTermInvestments,
    AccountsReceivable, Inventory,
    PropertyPlantEquipment, Goodwill, IntangibleAssets,
    CurrentLiabilities, NonCurrentLiabilities,
    LongTermDebt, RetainedEarnings,

    // --- Level 2: Income Statement sub-elements ---
    CostOfRevenue, GrossProfit,
    SellingGeneralAdmin, ResearchDevelopment,
    InterestExpense, OperatingIncome,
    EarningsPerShareBasic, EarningsPerShareDiluted,

    // --- Level 2: Cash Flow sub-elements ---
    DepreciationAmortization, CapitalExpenditures,
    DividendsPaid, ShareRepurchases,
    ShareBasedCompensation,
}
```

These are all standard US-GAAP taxonomy concepts served by the SEC JSON API. Verified against Apple and Alphabet company facts data.

---

## 1) Parsing

### Input / Output

**Input**: Raw JSON body (from SEC API response, already fetched by the `sec` pipeline)

**Output**: A collection of `RawObservation` — one per data point, fully typed, not yet resolved to canonical elements.

### Data Types

```
RawObservation {
    namespace:     Namespace          // UsGaap | Dei | Srt | ...
    concept_name:  String             // "Revenues", "Assets", etc.
    value:         i64
    unit:          Unit               // Usd | Shares | Pure
    period:        Period             // Instant(date) | Duration(start, end)
    frame:         Option<Frame>
    provenance:    Provenance
}

Provenance {
    accession_number: AccessionNumber
    form:             Form            // 10-K | 10-Q | 10-K/A | ...
    fiscal_year:      FiscalYear
    fiscal_period:    FiscalPeriod    // FY | Q1 | Q2 | Q3
    filed_date:       Date
    period_end:       Date
}
```

The split reflects the two concerns in the SEC JSON's 9 keys:
- **Observation** = what was measured (`val`, `start`/`end`, `frame`, unit from parent concept)
- **Provenance** = where it came from (`accn`, `form`, `fy`, `fp`, `filed`, `end`)

The `RawObservation` owns its `Provenance` inline (embedded, not referenced). This keeps each observation self-contained. The duplication is negligible since provenance is only 6 small fields.

Later, when building the storage layer (fact store), this can be normalized to reference-by-`AccessionNumber` instead. That refactor is mechanical.

**NOTE**: These types already exist in the `sec` crate as `Observation` and `FilingSource` with this exact structure. They migrate into `xbrl::core` when the crate is extracted.

### Key Design Decisions

**Period type** — The SEC JSON doesn't explicitly label instant vs duration, but you can distinguish them: if a data point has only `end` (no `start`), it's an instant (balance sheet). If it has both `start` and `end`, it's duration (income statement, cash flow). The `frame` field also encodes this — frames ending in `I` are instants. This distinction is critical because it maps directly to which financial statement a fact belongs to.

**Namespaces** — The JSON has `dei` and `us-gaap` at the top level under `facts`. Parse both. Some companies also have `srt`, `invest`, `ffd`, `ecd`. The namespace determines where to look during resolution (shares outstanding lives in `dei`, most financials in `us-gaap`).

**Amendments** — Multiple data points for the same concept + period will exist when a company files an amendment (10-K/A). The parser should emit ALL of them. Deduplication is a resolution concern, not a parsing concern.

**Concept name instability** — Concept names shift over time due to FASB taxonomy updates. Example: Apple's Revenue was `SalesRevenueNet` (2009-2017), `Revenues` (2018), then `RevenueFromContractWithCustomerExcludingAssessedTax` (2019-2025). The alias system handles this — each canonical element has a prioritized alias list. First alias that matches wins. When two aliases both exist for the same period (overlapping transition years), the first alias in the priority list wins.

### Handling Different SEC API Schemas

The SEC has three relevant JSON APIs:
1. **Company Facts** (`/companyfacts/CIK{n}.json`) — all facts for one company
2. **Company Concept** (`/companyconcept/CIK{n}/{taxonomy}/{concept}.json`) — one concept for one company
3. **Frames** (`/frames/{taxonomy}/{concept}/{unit}/{period}.json`) — one concept across all companies

All three serve data points with the same 9 keys: `accn`, `end`, `filed`, `form`, `fp`, `frame`, `fy`, `start`, `val`. The JSON envelope differs but the data points are identical.

Handle with separate deserializers that all produce the same output:

```
xbrl::sec_api::company_facts    -> Vec<RawObservation>
xbrl::sec_api::company_concept  -> Vec<RawObservation>
xbrl::sec_api::frames           -> Vec<RawObservation>
```

The resolution engine doesn't care which API the observation came from.

---

## 2) Resolution

### ResolvedFact

> **[DECISION NEEDED]** Finalize the `ResolvedFact` type before implementation. Draft below.

```
ResolvedFact {
    canonical_name:    CanonicalElement
    value:             i64
    unit:              Unit
    period:            Period
    confidence:        Confidence        // Exact | Synonym | Derived | Computed
    resolution_path:   Vec<String>       // which concept names / derivation steps produced this
    source:            Vec<Provenance>   // one if direct, multiple if derived from several facts
}
```

The `resolution_path` is traceability — e.g., "Revenue resolved via `SalesRevenueNet` (Tier 2, Synonym)" or "Equity derived from `Assets - Liabilities` (Tier 3, Derived)".

### Deduplication Strategy

Same concept + same period can have multiple data points from different filings. Priority rules:
1. Amendment (10-K/A) always wins over original (10-K) for same period
2. Same period, multiple filings -> pick the one with the latest `filed_date`
3. Same period, same filing date, one has `frame` and one doesn't -> pick the one with `frame`

### Three Layers

**Layer 1 — Canonical Elements (SFAC 6)**

The root elements are the target. Everything resolves down to these.

```
Assets = Liabilities + Equity
Comprehensive Income = Revenue - Expenses + Gains - Losses
ΔEquity = Comprehensive Income + Investments by Owners - Distributions to Owners
```

**Layer 2 — Resolution (how to populate each element)**

Each element has a tiered resolution chain:

```
Element: "Total Assets"
  Tier 1  Direct:    Assets                              -> confidence: Exact
  Tier 2  Synonym:   AssetsCurrent + AssetsNoncurrent     -> confidence: Synonym
  Tier 3  Derived:   Liabilities + Equity                 -> confidence: Derived
  Tier 4  Linkbase:  Walk FASB calculation tree            -> confidence: Computed
```

First tier that resolves wins. Every resolved value carries which tier produced it.

**Layer 3 — Validation (invariants as constraints)**

Once elements are populated, enforce the identities:

```
Check:  Assets == Liabilities + Equity
Check:  Net Income == Revenue - Expenses + Gains - Losses
Check:  Operating CF + Investing CF + Financing CF ≈ ΔCash
```

A validation failure with all Tier 1 values = the company filed inconsistent data.
A validation failure with Tier 3 values = your derivation logic has a bug.

### The Key Insight

Resolution and validation are the same graph. The SFAC 6 identities define edges between elements. When resolving, you traverse edges to derive unknowns. When validating, you check that the edges hold. Same structure, two modes.

```
Nodes  = SFAC 6 Elements (what you extract)
Edges  = Invariants       (how they relate)
Mode 1 = Resolve          (fill unknowns from knowns)
Mode 2 = Validate         (verify all edges hold)
```

### Graph Structure

> **[DECISION NEEDED]** Define the graph data structure before implementation.
>
> Options:
> - Adjacency list with weighted edges (+1.0, -1.0) matching FASB linkbase format
> - Trait-based: both SFAC 6 invariants and FASB linkbase implement a common `ResolutionGraph` trait
> - Hardcoded invariant functions for Level 1, linkbase-driven for Level 2+
>
> This determines whether resolution and validation share actual code or just a conceptual model.

---

## 3) Validation

Data quality checks to validate that parsed data makes sense.

Three levels, applied in order of strictness:

1. **Completeness** — Are all required canonical elements present? Can't validate what's missing.
2. **Consistency** — Do the SFAC 6 identities hold? A structural violation (e.g., `Assets != Liabilities + Equity`).
3. **Precision** — Do sub-items sum exactly to their reported parent totals? A rounding or classification issue.

Specific checks include:
- Balance Sheet Identity holds (`Assets == Liabilities + Equity`)
- Roll-up items equal the sum of all sub-items
- Sign checks (Assets > 0, Liabilities > 0, Revenue > 0 for non-loss companies)
- Cross-statement consistency (Net Income in IS == Net Income in CF reconciliation)
- Rules inspired by DQC Rules (https://github.com/DataQualityCommittee/dqc_us_rules)

### Precision Threshold

For values computed by the resolution engine (Tier 2/3 derivations), precision must be exactly 100% — our math must be exact.

When cross-checking a company's own reported values (e.g., their reported `Assets` vs the sum of their reported `AssetsCurrent + AssetsNoncurrent`), allow a small tolerance. Companies report in thousands or millions, introducing ±1 rounding differences. Threshold: ±0.1% of the parent value.

Rule: **our math must be exact, their math gets a tolerance.**

---

## 4) Error Handling

Uses `thiserror` with `#[error("...")]` and `#[source]` chaining. Follows the same conventions as the `sec` crate:
- `[BracketedName]` prefix on every error segment
- `Caused by:` to chain to the next error (no quotes around inner errors)
- `Reason:` for leaf errors with free-form descriptions (quoted)
- No periods between segments — commas separate description from `Caused by:` / `Reason:`

### Error Hierarchy

```
ErrorKind
  ::Xbrl(XbrlErrorKind)
    ::FailedParsing(ParseErrorKind)
      ::InvalidJson { reason: String }
      ::MissingTopLevelKey { key: String }
      ::MissingNamespace { namespace: String }
      ::InvalidDataPoint { concept: String, reason: String }
    ::FailedValidation(ValidationErrorKind)
      ::IncompleteData { missing_elements: Vec<CanonicalElement> }
      ::InconsistentIdentity { invariant: Invariant, left_value: i64, right_value: i64 }
      ::ImpreciseRollup { parent: CanonicalElement, parent_value: i64, children_sum: i64, deviation_pct: f64, threshold_pct: f64 }
```

### Error Severity

- **`FailedParsing`** — Fatal. Bad input, cannot proceed.
- **`IncompleteData`** — Fatal. Missing required elements, cannot validate or produce statements.
- **`InconsistentIdentity`** — Hard error. A financial identity is broken.
- **`ImpreciseRollup`** — Warning when within tolerance, error when exceeded.

### Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("[Xbrl] {0}")]
    Xbrl(#[source] XbrlErrorKind),
}

#[derive(Debug, thiserror::Error)]
pub enum XbrlErrorKind {
    #[error("[FailedParsing] Caused by: {0}")]
    FailedParsing(#[source] ParseErrorKind),

    #[error("[FailedValidation] Caused by: {0}")]
    FailedValidation(#[source] ValidationErrorKind),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("[InvalidJson] Failed to parse JSON body, Reason: '{reason}'")]
    InvalidJson { reason: String },

    #[error("[MissingTopLevelKey] Expected key '{key}' in response")]
    MissingTopLevelKey { key: String },

    #[error("[MissingNamespace] Expected namespace '{namespace}' under 'facts'")]
    MissingNamespace { namespace: String },

    #[error("[InvalidDataPoint] Invalid data point for concept '{concept}', Reason: '{reason}'")]
    InvalidDataPoint { concept: String, reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationErrorKind {
    #[error("[IncompleteData] Missing required elements: {missing_elements}")]
    IncompleteData {
        missing_elements: Vec<CanonicalElement>,
    },

    #[error("[InconsistentIdentity] Invariant '{invariant}' violated, left={left_value}, right={right_value}")]
    InconsistentIdentity {
        invariant: Invariant,
        left_value: i64,
        right_value: i64,
    },

    #[error("[ImpreciseRollup] Roll-up mismatch for '{parent}', reported={parent_value}, computed={children_sum}, Reason: 'Deviation of {deviation_pct:.4}% exceeds threshold of {threshold_pct:.4}%'")]
    ImpreciseRollup {
        parent: CanonicalElement,
        parent_value: i64,
        children_sum: i64,
        deviation_pct: f64,
        threshold_pct: f64,
    },
}
```

### Invariant Enum

All financial relationships are invariants — the only difference is whether they must hold exactly or within a tolerance. `is_exact()` distinguishes the two.

Exact invariant violations produce `InconsistentIdentity` (hard error). Non-exact invariant violations produce `ImpreciseRollup` (tolerance-based).

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invariant {
    // Exact — SFAC 6 identities, must hold with zero tolerance
    BalanceSheetIdentity,        // Assets = Liabilities + Equity
    NetIncomeIdentity,           // Net Income = Revenue - Expenses + Gains - Losses
    ComprehensiveIncomeIdentity, // Comprehensive Income = Net Income + OCI
    EquityChangeIdentity,        // ΔEquity = CompIncome + Investments - Distributions

    // Non-exact — must hold within tolerance
    CashFlowReconciliation,     // Operating + Investing + Financing ≈ ΔCash
}

impl Invariant {
    pub const fn is_exact(&self) -> bool {
        matches!(
            self,
            Self::BalanceSheetIdentity
                | Self::NetIncomeIdentity
                | Self::ComprehensiveIncomeIdentity
                | Self::EquityChangeIdentity
        )
    }
}
```

### Example Error Chains

```
[Xbrl] [FailedValidation] Caused by: [InconsistentIdentity] Invariant 'BalanceSheetIdentity' violated, left=500000000, right=490000000

[Xbrl] [FailedValidation] Caused by: [ImpreciseRollup] Roll-up mismatch for 'Assets', reported=500000000, computed=499998000, Reason: 'Deviation of 0.0004% exceeds threshold of 0.0000%'
```

---

## 5) Financial Statement Output Types

> **[DECISION NEEDED]** Define output types. One generic `FinancialStatement` or typed per-statement?
>
> Option A — Typed:
> ```
> BalanceSheet      { entity, period: Instant,  facts: HashMap<CanonicalElement, ResolvedFact> }
> IncomeStatement   { entity, period: Duration, facts: HashMap<CanonicalElement, ResolvedFact> }
> CashFlowStatement { entity, period: Duration, facts: HashMap<CanonicalElement, ResolvedFact> }
> ```
>
> Option B — Generic:
> ```
> FinancialStatement { entity, period, statement_type, facts: HashMap<CanonicalElement, ResolvedFact> }
> ```
>
> Option A catches period-type mismatches at compile time (BS is always instant, IS/CF always duration).
> Option B is simpler and more extensible.

---

## 5) Data Modeling — Level 2 Sub-Elements

SFAC 6 gives top-level identities. A useful financial dataset needs the breakdown beneath them.

**Level 1 — SFAC 6 backbone:**
Assets, Liabilities, Equity, Revenue, Expenses, Net Income, Operating CF, Investing CF, Financing CF

**Level 2 — Key sub-elements:**

Balance Sheet:
- Cash & Cash Equivalents
- Short-term Investments
- Accounts Receivable
- Inventory
- Current Assets (subtotal)
- PP&E (Property, Plant & Equipment)
- Goodwill & Intangibles
- Current Liabilities (subtotal)
- Long-term Debt
- Retained Earnings

Income Statement:
- Cost of Goods Sold / Cost of Revenue
- Gross Profit
- SG&A (Selling, General & Administrative)
- R&D Expense
- Interest Expense
- EPS Basic / Diluted

Cash Flow:
- Depreciation & Amortization
- Capital Expenditures
- Dividends Paid
- Share Repurchases

These are all standard base taxonomy concepts. The same FASB calculation linkbase defines how they sum into the SFAC 6 parents. The SEC JSON API reports them — verified with Apple and Alphabet data.

The model stays the same — just more nodes in the graph. Each sub-element gets the same tiered resolution (direct -> synonym -> derived).

---

## Open Questions

### Resolved

**Fact sets** — Natural grouping: one entity + one period + one form type = one fact set. The parser emits raw observations; the resolution engine groups them into fact sets before resolving.

**Roll-ups** — Not a parsing concern. The parser emits flat observations. The tree structure is in `xbrl::us_gaap::taxonomy` and applied during resolution.

**Period vs point-in-time** — Solved at parse time by the `Period` type. Every observation carries `Instant(date)` or `Duration(start, end)`.

**Member aggregations / segments** — The SEC Company Facts JSON API does NOT include segment data, dimensional context, or company extension concepts. All three JSON APIs serve only consolidated, un-dimensioned facts from standard taxonomies. Segment data lives in raw XBRL instance documents (`_htm.xml`) in EDGAR filing archives and requires a separate parsing pipeline. This confirms two pipelines:
1. Company Facts API -> consolidated financial statements (Phase 1)
2. Raw XBRL instance documents -> segment breakdowns (Phase 2)

**Overlapping concept names** — First alias in the priority list wins. Already how `ConceptDefinition` works.

**Crate boundary** — `xbrl::sec_api` takes `&str` or `&serde_json::Value`. The `sec` crate owns HTTP.

### Deferred (not needed for Phase 1)

**Filing amendments to existing data storage** — Handle at the storage layer, not the parsing/resolution layer. The resolution layer's deduplication rules (amendment wins over original) handle correctness. Storage-level amendment handling is a fact store concern.

**Adjustments (change in accounting policy / correction of error)** — Companies restate prior periods when policies change. The SEC JSON API includes restated values as new data points with newer `filed_date`. The deduplication strategy (latest `filed_date` wins) handles this automatically.

**Variance (budgeted vs actual)** — Not applicable. SEC filings contain actuals only, not budgets. Not needed.

**Fragments** — Not applicable for the JSON API pipeline. Fragments are an XBRL instance document concept relevant only to Phase 2 (raw XBRL parsing).

**Roll forwards** — Relevant for Changes in Equity statement. Deferred until that statement type is implemented.

---

## Decisions Needed Before Implementation

1. **CanonicalElement enum** — Finalize the full enumeration (draft above)
2. **ResolvedFact type** — Finalize fields, especially `resolution_path` representation
3. **Graph structure** — Adjacency list vs trait-based vs hardcoded functions
4. **Financial statement output types** — Typed per-statement vs generic

These four decisions shape the core API surface. Everything else can be resolved during implementation.

---

## Follow-Up: Migrate `sec` Crate to Consume `xbrl` Types

The `sec` crate currently has its own financial domain types (`CompanyData`, `CompanyFact`,
`Observation`, `FilingSource`, `ConceptDefinition`) that duplicate what `xbrl` now provides.

This requires two follow-up tickets:

### Ticket 1: Design the integration pathway (SPIKE)

Research and decide:
- How `Cik` (sec-specific) interacts with `EntityName` (xbrl) in `CompanyData`
- Whether `CompanyData.facts` key becomes `CanonicalElement` or stays as a definition reference
- Whether `sec` types become re-exports of `xbrl` types or thin wrappers with `From` impls
- How to handle the `&'static ConceptDefinition` reference pattern (sec's constants vs xbrl's)
- Migration strategy that keeps the `sec` crate compiling at every step
- Impact on downstream consumers (`CreateFinancialStatements`, pipeline tests, binaries)

Output: a design doc section or ADR with the recommended approach.

### Ticket 2: Implement the migration

Based on the design from Ticket 1:
1. Replace `sec::shared::financial::Observation` / `FilingSource` with `xbrl` equivalents
2. Replace `sec::shared::financial::ConceptDefinition` with `xbrl::us_gaap::mappings::ConceptDefinition`
3. Update `CompanyData` to use xbrl types for its key and value types
4. Update `ParseCompanyFacts::compute_output_data_async()` to delegate to `xbrl::sec_api::company_facts::parse()`
5. Remove the inline `resolve_concept`, `parse_observation`, `build_period`, `build_filing_source` from `sec`
6. Remove duplicated type definitions from `sec::shared::financial/`

**Blocker for Ticket 2**: `CompanyData` uses `Cik` (sec-specific) and `&'static ConceptDefinition` as
HashMap keys (pointing to sec's constants). Both need to change before the delegation works.
This touches the `sec` crate's public API and downstream consumers.

---

## Key Findings From Investigation

- All three SEC JSON APIs serve data points with identical 9-key schema
- No segment data, no dimensional context, no company extensions in any JSON API
- Concept names shift over time (FASB taxonomy updates) — alias lists must cover historical names
- 15-17 years of history available for most S&P 500 companies
- The FASB calculation linkbase is universal (one tree, same for every company)
- Sub-item data (COGS, SG&A, R&D, PP&E, etc.) is available in the JSON API — sufficient for Level 2
- Segment data requires raw XBRL instance documents — separate pipeline
