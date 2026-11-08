//! # Finding
//!
//! Provides [`Finding`], a fact about the data that a step reports without failing the run.

use chrono::NaiveDate;

use super::candidate::SeriesKey;
use super::concept::NamespaceFamily;
use super::elements::CanonicalElement;
use super::fact::{Dimensions, FactKey};
use super::filing::{EntityId, FilingId};
use super::fiscal_period::FiscalPeriod;
use super::invalid_value::InvalidValue;
use super::unit::Unit;

/// A fact about the data that a step reports without failing the run.
///
/// A parser returns findings next to its facts. Later steps add their own variants.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// The filing holds facts of a namespace family that no plug-in declares. The parser keeps the
    /// facts with the full Notes `version` or URI as family and version. Such facts do not join
    /// between sources or across releases until a plug-in declares the family.
    UndeclaredFamily {
        /// The filing.
        filing: FilingId,
        /// The family, for example `xyz/2027`.
        family: NamespaceFamily,
    },
    /// The parser could not map a row of the source into a fact, so it dropped the row.
    UnmappedRow {
        /// The filing.
        filing: FilingId,
        /// The table and the primary key of the row.
        row: String,
        /// The rule the row breaks.
        reason: InvalidValue,
    },
    /// At least one copy rank of a value key disagrees with rank 0, so the copies are separate
    /// facts. `core/dedup/more-precise@1` keeps every rank and reports the conflict.
    Conflict {
        /// The entity the facts are about.
        entity: EntityId,
        /// The value key of rank 0. It names the filing, the as-is concept, and the period.
        key: FactKey,
        /// The ranks that disagree with rank 0, in ascending order.
        ranks: Vec<u16>,
        /// The standard concept of the as-is concept, if a plug-in maps it.
        standard_concept: Option<CanonicalElement>,
    },
    /// The series has no value for an expected period between its first and its last value.
    Gap {
        /// The entity the series is about.
        entity: EntityId,
        /// The standard concept of the series.
        concept: CanonicalElement,
        /// The unit of the series.
        unit: Unit,
        /// The axis and member pairs of the series.
        dimensions: Dimensions,
        /// The end of the expected period.
        period_end: NaiveDate,
        /// The expected fiscal period, for example `Q2`, if it is known.
        fiscal_period: Option<FiscalPeriod>,
        /// Why the value is missing, if the reason is known.
        reason: Option<GapReason>,
    },
    /// Two values of one series have the same length, and their periods overlap without being
    /// the same period.
    Overlap {
        /// The key of the value with the earlier end.
        first: SeriesKey,
        /// The key of the value with the later end.
        second: SeriesKey,
    },
    /// An older filing reported another value for the standardized value of this key or for one
    /// of its inputs.
    Restatement {
        /// The key of the restated value.
        key: SeriesKey,
    },
}

/// The known reason of a [`Finding::Gap`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapReason {
    /// The copy ranks of the newest filing disagree, so dedup blocks the value. A
    /// [`Finding::Conflict`] names the ranks.
    Conflict,
    /// The entity changed its fiscal year end next to the expected period.
    FiscalYearChange,
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_finding() {
        implements_auto_traits::<Finding>();
    }

    #[test]
    const fn should_implement_auto_traits_for_gap_reason() {
        implements_auto_traits::<GapReason>();
    }
}
