---
source: FASB US GAAP 2026 Taxonomy (xbrl.fasb.org/us-gaap/2026/stm/)
last-verified: 2026-11-08
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

## How to Apply (Tier 4 Computation)

When a company does not report a parent concept, compute it by summing its children:

1. If the filing reports the parent concept, use it (Tier 1 or 2)
2. If not, look up the children of the parent in the calculation arcs of one role
3. Check that the filing reports ALL children
4. Compute: `parent = Σ(child_i × weight_i)`
5. Store with `confidence: Computed`

Example: Amazon does not report `Liabilities` but reports `LiabilitiesCurrent` and `LiabilitiesNoncurrent`. Compute: `Liabilities = LiabilitiesCurrent(+1) + LiabilitiesNoncurrent(+1)`
