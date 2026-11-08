//! # Candidate
//!
//! Provides [`Candidate`], the value that one filing gives for one standard concept, before
//! `core/dedup/newest-filing@1` picks one value across filings.

use bigdecimal::BigDecimal;
use chrono::NaiveDate;

use super::confidence::Confidence;
use super::elements::CanonicalElement;
use super::fact::{Decimals, Dimensions, FactKey, FilingSource};
use super::filing::EntityId;
use super::period::Period;
use super::release::TaxonomyRelease;
use super::rule::RuleId;
use super::unit::Unit;

/// The value that one filing gives for one standard concept and one period.
///
/// Steps 1 and 2 of the dedup order produce at most one candidate per standard concept, period,
/// and filing. Step 3 picks one candidate across filings and turns it into a
/// [`StandardizedFact`](super::standardized::StandardizedFact).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Candidate {
    /// The filing and when it was filed.
    pub source: FilingSource,
    /// The entity the value is about.
    pub entity: EntityId,
    /// The standard concept.
    pub concept: CanonicalElement,
    /// The instant or duration the value covers.
    pub period: Period,
    /// The unit.
    pub unit: Unit,
    /// The axis and member pairs.
    pub dimensions: Dimensions,
    /// The value, or the mark that a conflict blocks it.
    pub value: CandidateValue,
    /// The rule that picked the as-is concept, for example `us-gaap/alias/revenue@1`.
    pub rule: RuleId,
    /// The release of the plug-in that resolved the concept.
    pub release: TaxonomyRelease,
    /// The value keys of the as-is facts. A blocked candidate names every conflicting rank.
    pub inputs: Vec<FactKey>,
}

impl Candidate {
    /// Returns the key of the series this candidate belongs to.
    #[must_use]
    pub fn series_key(&self) -> SeriesKey {
        SeriesKey {
            entity: self.entity.clone(),
            concept: self.concept,
            period_end: self.period.end(),
            quarters: self.period.quarters(),
            unit: self.unit.clone(),
            dimensions: self.dimensions.clone(),
        }
    }
}

/// The value of a [`Candidate`]: a number, or blocked by a conflict.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum CandidateValue {
    /// The filing reports one value for the concept.
    Reported {
        /// The value.
        value: BigDecimal,
        /// The precision of the value, if the source gives it.
        decimals: Option<Decimals>,
        /// The tier that produced the value.
        confidence: Confidence,
    },
    /// The copy ranks of the concept disagree, so the filing gives no value. A
    /// [`Finding::Conflict`](super::finding::Finding::Conflict) names the ranks.
    Blocked,
}

/// The key on which `core/dedup/newest-filing@1` matches candidates of different filings.
///
/// The key holds the exact end and the length in quarters, as the value key does, because the
/// Notes data set often gives no exact start.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct SeriesKey {
    /// The entity.
    pub entity: EntityId,
    /// The standard concept.
    pub concept: CanonicalElement,
    /// The exact end of the period.
    pub period_end: NaiveDate,
    /// The length in quarters.
    pub quarters: u16,
    /// The unit.
    pub unit: Unit,
    /// The axis and member pairs.
    pub dimensions: Dimensions,
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_candidate() {
        implements_auto_traits::<Candidate>();
    }

    #[test]
    const fn should_implement_auto_traits_for_candidate_value() {
        implements_auto_traits::<CandidateValue>();
    }

    #[test]
    const fn should_implement_auto_traits_for_series_key() {
        implements_auto_traits::<SeriesKey>();
    }
}
