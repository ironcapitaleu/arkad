//! # Raw Observation
//!
//! A single data point extracted from the SEC JSON API before concept resolution.

use super::frame::Frame;
use super::namespace::Namespace;
use super::period::Period;
use super::provenance::Provenance;
use super::unit::Unit;

/// A raw financial data point before concept resolution.
///
/// Contains the measured value, its context (period, unit, frame),
/// the XBRL concept it was tagged with, and full provenance metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RawObservation {
    /// The taxonomy namespace this concept belongs to.
    namespace: Namespace,
    /// The original XBRL concept tag name (e.g., `"Revenues"`).
    concept_name: String,
    /// The reported numeric value.
    value: i64,
    /// The unit of measurement.
    unit: Unit,
    /// The time period this measurement covers.
    period: Period,
    /// The SEC XBRL frame identifier, if present.
    frame: Option<Frame>,
    /// Filing metadata tracking where this data point came from.
    provenance: Provenance,
}

impl RawObservation {
    /// Creates a new [`RawObservation`] from its components.
    #[must_use]
    pub fn new(
        namespace: Namespace,
        concept_name: impl Into<String>,
        value: i64,
        unit: Unit,
        period: Period,
        frame: Option<Frame>,
        provenance: Provenance,
    ) -> Self {
        Self {
            namespace,
            concept_name: concept_name.into(),
            value,
            unit,
            period,
            frame,
            provenance,
        }
    }

    /// Returns the taxonomy namespace.
    #[must_use]
    pub const fn namespace(&self) -> Namespace {
        self.namespace
    }

    /// Returns the XBRL concept name.
    #[must_use]
    pub fn concept_name(&self) -> &str {
        &self.concept_name
    }

    /// Returns the reported value.
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

    /// Returns the frame, if present.
    #[must_use]
    pub const fn frame(&self) -> Option<Frame> {
        self.frame
    }

    /// Returns the provenance metadata.
    #[must_use]
    pub const fn provenance(&self) -> &Provenance {
        &self.provenance
    }
}
