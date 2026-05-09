//! # Resolved Fact
//!
//! A financial data point after concept resolution, carrying its canonical
//! element identity, confidence level, and full traceability.

use super::confidence::Confidence;
use super::elements::CanonicalElement;
use super::period::Period;
use super::provenance::Provenance;
use super::unit::Unit;

/// A resolved financial fact with full traceability.
///
/// Produced by the resolution engine after matching raw observations
/// to canonical elements. Every resolved fact carries metadata about
/// how it was obtained and from which source filings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResolvedFact {
    /// The canonical financial element this fact represents.
    canonical_element: CanonicalElement,
    /// The resolved numeric value.
    value: i64,
    /// The unit of measurement.
    unit: Unit,
    /// The time period this measurement covers.
    period: Period,
    /// Which resolution tier produced this value.
    confidence: Confidence,
    /// The resolution steps that produced this value (for traceability).
    resolution_path: Vec<String>,
    /// The source filing(s) this value was derived from.
    source: Vec<Provenance>,
}

impl ResolvedFact {
    /// Creates a new [`ResolvedFact`] from its components.
    #[must_use]
    pub const fn new(
        canonical_element: CanonicalElement,
        value: i64,
        unit: Unit,
        period: Period,
        confidence: Confidence,
        resolution_path: Vec<String>,
        source: Vec<Provenance>,
    ) -> Self {
        Self {
            canonical_element,
            value,
            unit,
            period,
            confidence,
            resolution_path,
            source,
        }
    }

    /// Returns the canonical element.
    #[must_use]
    pub const fn canonical_element(&self) -> CanonicalElement {
        self.canonical_element
    }

    /// Returns the resolved value.
    #[must_use]
    pub const fn value(&self) -> i64 {
        self.value
    }

    /// Returns the unit of measurement.
    #[must_use]
    pub const fn unit(&self) -> Unit {
        self.unit
    }

    /// Returns the time period.
    #[must_use]
    pub const fn period(&self) -> Period {
        self.period
    }

    /// Returns the confidence level.
    #[must_use]
    pub const fn confidence(&self) -> Confidence {
        self.confidence
    }

    /// Returns the resolution path (traceability).
    #[must_use]
    pub fn resolution_path(&self) -> &[String] {
        &self.resolution_path
    }

    /// Returns the source provenance records.
    #[must_use]
    pub fn source(&self) -> &[Provenance] {
        &self.source
    }
}
