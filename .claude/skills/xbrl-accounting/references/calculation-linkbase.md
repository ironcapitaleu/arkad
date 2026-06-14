---
source: FASB US GAAP 2026 Taxonomy (xbrl.fasb.org/us-gaap/2026/stm/)
last-verified: 2026-06-14
update-frequency: annually
taxonomy-year: 2026
---

# Calculation Linkbase — Key Derivation Relationships

Curated subset of FASB calculation linkbase arcs relevant to the `CanonicalElement` set. Source: `us-gaap-stm-*-cal-2026.xml` files.

## Income Statement (from `us-gaap-stm-soi-cal-2026.xml`)

```text
GrossProfit:
  + Revenues
  - CostOfRevenue

OperatingIncomeLoss:
  + GrossProfit
  - OperatingExpenses
  + GovernmentAssistanceOperatingIncome
  + OtherOperatingIncomeExpenseNet

IncomeLossFromContinuingOperationsBeforeIncomeTaxes...:
  + IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterest...
  + IncomeLossFromEquityMethodInvestments

NetIncomeLoss:
  + ProfitLoss
  - NetIncomeLossAttributableToNoncontrollingInterest
```

Key insight: `OperatingIncomeLoss` and `IncomeLossFromContinuingOperationsBeforeIncomeTaxes` are DIFFERENT levels. Operating Income is BEFORE interest/non-operating items. Pre-tax income is AFTER.

## Balance Sheet (from `us-gaap-stm-sfp-cls-cal-2026.xml`)

```text
Assets:
  + AssetsCurrent
  + AssetsNoncurrent

Liabilities:
  + LiabilitiesCurrent
  + LiabilitiesNoncurrent

LiabilitiesAndStockholdersEquity:
  + Liabilities
  + CommitmentsAndContingencies
  + TemporaryEquityCarryingAmount...
  + StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest

StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest:
  + StockholdersEquity
  + MinorityInterest

StockholdersEquity:
  + PreferredStockValue
  - PreferredStockSharesSubscribedButUnissued...
  + CommonStockValue
  - TreasuryStockValue
  + AdditionalPaidInCapital
  + AccumulatedOtherComprehensiveIncomeLossNetOfTax
  + RetainedEarningsAccumulatedDeficit
  ... (additional components)
```

Key derivations:

- `Assets = AssetsCurrent + AssetsNoncurrent`
- `Liabilities = LiabilitiesCurrent + LiabilitiesNoncurrent`
- Balance sheet equation: `Assets = LiabilitiesAndStockholdersEquity`

## Cash Flow — Indirect Method (from `us-gaap-stm-scf-indir-cal-2026.xml`)

```text
NetCashProvidedByUsedInOperatingActivities:
  + ProfitLoss
  + AdjustmentsToReconcileNetIncomeLossToCashProvidedByUsedInOperatingActivities

NetCashProvidedByUsedInInvestingActivities:
  + NetCashProvidedByUsedInInvestingActivitiesContinuingOperations
  + CashProvidedByUsedInInvestingActivitiesDiscontinuedOperations

NetCashProvidedByUsedInFinancingActivities:
  + NetCashProvidedByUsedInFinancingActivitiesContinuingOperations
  + CashProvidedByUsedInFinancingActivitiesDiscontinuedOperations

CashCashEquivalentsRestrictedCash...PeriodIncreaseDecreaseIncludingExchangeRateEffect:
  + CashCashEquivalents...PeriodIncreaseDecreaseExcludingExchangeRateEffect
  + EffectOfExchangeRateOnCash...
```

## How to Apply (Tier 3 Derivation)

When a company doesn't report a parent concept, derive it by summing children:

1. Check if parent concept exists → use it (Tier 1/2)
2. If not, check if ALL required children exist in the filing
3. If yes, compute: `parent = Σ(child_i × weight_i)`
4. Store with `confidence: Derived`

Example: Amazon doesn't report `Liabilities` but reports `LiabilitiesCurrent` + `LiabilitiesNoncurrent`. Derive: `Liabilities = LiabilitiesCurrent(+1) + LiabilitiesNoncurrent(+1)`
