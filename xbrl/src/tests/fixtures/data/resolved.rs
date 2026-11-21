//! # Resolved Fixture Filings
//!
//! Runs the rules of one filing on fixture Notes filings with the US GAAP plug-in:
//! `core/dedup/more-precise@1`, then tiers 1 and 2. The acceptance tests start from the result.

use chrono::NaiveDate;

use super::notes::parse_notes_fixture_filing;
use crate::core::alias::resolve_aliases;
use crate::core::candidate::Candidate;
use crate::core::dedup::{Series, more_precise, newest_filing};
use crate::core::elements::{
    BalanceSheetConcept, CanonicalElementKind, CashFlowConcept, IncomeStatementConcept,
};
use crate::core::filing::Filing;
use crate::core::finding::Finding;
use crate::core::period::Period;
use crate::core::standardized::StandardizedFact;
use crate::core::taxonomy::registry::{Plugin, Registry};
use crate::us_gaap::UsGaap;

/// The 11 roll-ups of the MVP Decision Spike that map to a standard concept.
pub const SPIKE_ROLL_UPS: [CanonicalElementKind; 11] = [
    BalanceSheetConcept::Assets.element(),
    BalanceSheetConcept::Liabilities.element(),
    BalanceSheetConcept::Equity.element(),
    BalanceSheetConcept::LiabilitiesAndEquity.element(),
    IncomeStatementConcept::Revenue.element(),
    IncomeStatementConcept::NetIncome.element(),
    IncomeStatementConcept::NetIncomeAttributableToParent.element(),
    CashFlowConcept::OperatingCashFlow.element(),
    CashFlowConcept::InvestingCashFlow.element(),
    CashFlowConcept::FinancingCashFlow.element(),
    CashFlowConcept::NetChangeInCash.element(),
];

/// The most days between the end of a value and the end of its fiscal year. A 52-53 week year ends
/// up to 6 days away from the end of its month.
const YEAR_END_TOLERANCE_DAYS: i64 = 7;

/// The length of a fiscal year, in quarters.
const YEAR_LENGTH: u16 = 4;

/// The filings, candidates, and findings of fixture Notes filings of one entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFilings {
    /// One filing per accession, in the order of the accessions.
    pub filings: Vec<Filing>,
    /// The candidates of every filing, in the order of the accessions.
    pub candidates: Vec<Candidate>,
    /// The findings of tiers 1 and 2 of every filing, in the order of the accessions.
    pub findings: Vec<Finding>,
}

impl ResolvedFilings {
    /// Returns the series of the candidates after `core/dedup/newest-filing@1`.
    #[must_use]
    pub fn series(&self) -> Series {
        newest_filing(&self.candidates)
    }
}

/// Parses each fixture Notes filing of `accessions`, and runs `core/dedup/more-precise@1` and
/// tiers 1 and 2 on it with [`UsGaap`].
///
/// # Panics
///
/// Panics if no compressed fixture filing has one of the accessions.
#[must_use]
pub fn resolve_fixture_filings(accessions: &[&str]) -> ResolvedFilings {
    let plugin = UsGaap::new();
    let registry = Registry::with_plugins(vec![Plugin::WithSegmentAxes(&plugin)])
        .expect("Given one plug-in with distinct families, the creation should always succeed");
    let mut resolved = ResolvedFilings {
        filings: Vec::new(),
        candidates: Vec::new(),
        findings: Vec::new(),
    };
    for accession in accessions {
        let parsed = parse_notes_fixture_filing(accession);
        let filing_candidates = resolve_aliases(&more_precise(&parsed.facts), &registry);
        resolved.filings.push(parsed.filing);
        resolved.candidates.extend(filing_candidates.candidates);
        resolved.findings.extend(filing_candidates.findings);
    }
    resolved
}

/// Returns the first fact of `facts` with the concept `concept` and no dimensions for the fiscal
/// year that ends at `year_end`: an instant within 7 days of it, or a year that ends within 7 days
/// of it.
pub fn fiscal_year_value<'a>(
    facts: impl IntoIterator<Item = &'a StandardizedFact>,
    concept: CanonicalElementKind,
    year_end: NaiveDate,
) -> Option<&'a StandardizedFact> {
    facts.into_iter().find(|fact| {
        fact.concept == concept
            && fact.dimensions.is_empty()
            && (fact.period.end() - year_end).num_days().abs() <= YEAR_END_TOLERANCE_DAYS
            && (matches!(fact.period, Period::Instant { .. })
                || fact.period.quarters() == YEAR_LENGTH)
    })
}
