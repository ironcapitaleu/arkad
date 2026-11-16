//! # Finding
//!
//! Provides [`Finding`], a fact about the data that a step reports without failing the run.

use std::fmt;

use chrono::NaiveDate;

use super::candidate::SeriesKey;
use super::concept::NamespaceFamily;
use super::confidence::Confidence;
use super::elements::{CanonicalElementKind, GroupName};
use super::fact::{Dimensions, FactKey};
use super::filing::{EntityId, FilingId};
use super::fiscal_period::FiscalPeriod;
use super::invalid_value::InvalidValue;
use super::period::Period;
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
    /// At least one copy rank of a value key conflicts with rank 0: a copy disagrees with it, or a
    /// rank with another exact start lies nearer to a whole number of quarters.
    /// `core/dedup/more-precise@1` keeps every rank and reports the conflict.
    Conflict {
        /// The entity the facts are about.
        entity: EntityId,
        /// The value key of rank 0. It names the filing, the as-is concept, and the period.
        key: FactKey,
        /// The ranks that conflict with rank 0, in ascending order.
        ranks: Vec<u16>,
        /// The standard concept of the as-is concept, if a plug-in maps it. If two alias lists of
        /// the plug-in name the concept, this holds the first one, in the order of `aliases()`.
        standard_concept: Option<CanonicalElementKind>,
    },
    /// The series has no value for an expected period between its first and its last value.
    Gap {
        /// The entity the series is about.
        entity: EntityId,
        /// The standard concept of the series.
        concept: CanonicalElementKind,
        /// The unit of the series.
        unit: Unit,
        /// The axis and member pairs of the series.
        dimensions: Dimensions,
        /// The end of the expected period.
        period_end: NaiveDate,
        /// The expected fiscal period, for example `Q2`, if it is known.
        fiscal_period: Option<FiscalPeriod>,
        /// The length of the expected period in quarters, as in the value key. A gap of a timeline
        /// of instants has 0.
        quarters: u16,
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
    /// The period of a standardized value hits a [`PeriodSignal`]. In the measured Notes files,
    /// such a period is a typo of the filer. The value keeps its period as filed.
    SuspiciousPeriod {
        /// The entity the value is about.
        entity: EntityId,
        /// The value key of the as-is fact. It names the filing, the as-is concept, and the period.
        key: FactKey,
        /// The standard concept of the value.
        standard_concept: CanonicalElementKind,
        /// The signals the period hits, in the order of [`PeriodSignal`]. Never empty.
        signals: Vec<PeriodSignal>,
        /// The newest other filing that reports the same value for a plausible period, or `None`.
        confirmed_by: Option<FilingId>,
    },
    /// The members of a closed group of the roll-up tree do not sum to the parent, or `Assets`
    /// differs from `LiabilitiesAndEquity`. The difference exceeds 0.1% of the largest absolute
    /// value of the check.
    RollUpMismatch {
        /// The entity the values are about.
        entity: EntityId,
        /// The period of the first value. The values share the end, the length in quarters,
        /// and the period type.
        period: Period,
        /// The unit of the values.
        unit: Unit,
        /// The axis and member pairs of the values.
        dimensions: Dimensions,
        /// The concepts with a value: the parent, then the start of a step, then the children in
        /// the order of their enum. For the balance sheet identity, `Assets`, then
        /// `LiabilitiesAndEquity`.
        concepts: Vec<CanonicalElementKind>,
        /// The check that failed.
        reason: RollUpCheck,
        /// The weakest confidence of the values.
        confidence: Confidence,
    },
    /// The other line of a breakdown of parts falls below zero by more than 0.1% of the largest
    /// absolute value of the group. Two children overlap, or an alias list holds a wider concept.
    NegativeOther {
        /// The entity the values are about.
        entity: EntityId,
        /// The period of the first value. The values share the end, the length in quarters,
        /// and the period type.
        period: Period,
        /// The unit of the values.
        unit: Unit,
        /// The axis and member pairs of the values.
        dimensions: Dimensions,
        /// The concepts with a value: the parent, then the children in the order of their enum.
        concepts: Vec<CanonicalElementKind>,
        /// The group whose other line falls below zero.
        reason: GroupName,
        /// The weakest confidence of the values.
        confidence: Confidence,
    },
    /// Two facts that the statement counts in one group of the roll-up tree have different exact
    /// starts, such as a quarter and a 4-month stub. The group gets no check, no derived value,
    /// and no other line.
    MixedStarts {
        /// The entity the values are about.
        entity: EntityId,
        /// The period of the first value of the slot. The values share the end, the length in
        /// quarters, and the period type.
        period: Period,
        /// The unit of the values.
        unit: Unit,
        /// The axis and member pairs of the values.
        dimensions: Dimensions,
        /// The concepts with a value: the parent, then the start of a step, then the children in
        /// the order of their enum. A member with no value gives way to the members of the group
        /// that makes it, in the order of their own table.
        concepts: Vec<CanonicalElementKind>,
        /// The group that mixes the starts.
        reason: GroupName,
    },
    /// The parts of a sum of a disclosure do not add up to its total. The difference exceeds the
    /// tolerance of the sum. The tolerance is in basis points of the largest absolute value of the
    /// check.
    SumMismatch {
        /// The entity the values are about.
        entity: EntityId,
        /// The period of the total, such as the instant of a maturity ladder.
        period: Period,
        /// The unit of the values.
        unit: Unit,
        /// The axis and member pairs of the values.
        dimensions: Dimensions,
        /// The total, then the parts in the order of their enum.
        concepts: Vec<CanonicalElementKind>,
        /// The name of the sum, such as `debt-maturities`.
        reason: &'static str,
        /// The weakest confidence of the values.
        confidence: Confidence,
    },
}

/// The check of the roll-up tree that a [`Finding::RollUpMismatch`] fails.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RollUpCheck {
    /// The check of a closed group: the start and the weighted children equal the parent.
    Group(GroupName),
    /// The balance sheet identity: `Assets` equals `LiabilitiesAndEquity`.
    BalanceSheet,
}

impl RollUpCheck {
    /// Returns the name of the check in findings: the name of the group, such as `liabilities`,
    /// or `balance-sheet`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Group(group) => group.as_str(),
            Self::BalanceSheet => "balance-sheet",
        }
    }
}

impl fmt::Display for RollUpCheck {
    /// Writes the name of the check, such as `liabilities` or `balance-sheet`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The rule that a [`Finding::SuspiciousPeriod`] hits.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodSignal {
    /// The period starts before 1900. For an instant, the date lies before 1900.
    StartBeforeEarliestYear,
    /// The period ends more than 100 years before or after the period of report of its filing.
    EndFarFromReport,
}

/// The known reason of a [`Finding::Gap`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapReason {
    /// The copy ranks of the newest filing disagree, so dedup blocks the value. A
    /// [`Finding::Conflict`] names the ranks.
    Conflict,
    /// The expected period lies in the 12 months after the latest period of report under the old
    /// fiscal year end.
    FiscalYearChange,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::elements::statement::balance_sheet::BalanceSheetGroup;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_finding() {
        implements_auto_traits::<Finding>();
    }

    #[test]
    const fn should_implement_auto_traits_for_gap_reason() {
        implements_auto_traits::<GapReason>();
    }

    #[test]
    const fn should_implement_auto_traits_for_period_signal() {
        implements_auto_traits::<PeriodSignal>();
    }

    #[test]
    const fn should_implement_auto_traits_for_roll_up_check() {
        implements_auto_traits::<RollUpCheck>();
    }

    #[test]
    fn should_write_the_group_name_when_the_check_is_a_closed_group() {
        let check = RollUpCheck::Group(BalanceSheetGroup::Liabilities.name());

        let expected_result = "liabilities";

        let result = check.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_balance_sheet_when_the_check_is_the_balance_sheet_identity() {
        let check = RollUpCheck::BalanceSheet;

        let expected_result = "balance-sheet";

        let result = check.to_string();

        assert_eq!(result, expected_result);
    }
}
