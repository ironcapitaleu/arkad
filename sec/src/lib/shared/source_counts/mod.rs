//! # Source Counts
//!
//! Provides [`SourceCounts`], the source rows of one filing and the rows that the parse dropped,
//! by [`DropReason`].
//!
//! The writer adds the counts of each filing into `source_rows` and `dropped_rows` of the
//! `as_is_facts` unit. A row count check then requires that the source rows minus the dropped
//! rows equal the rows of the unit.

use std::collections::BTreeMap;
use std::fmt;

use xbrl::core::finding::Finding;

/// Why the parse dropped a source row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DropReason {
    /// The parser could not map the row into a fact, and gave an `UnmappedRow` finding, as ADR
    /// 0041 sets.
    UnmappedRow,
    /// The row repeats a fact of an earlier row in key, value, and `decimals`, so the parser kept
    /// one fact for both and gave no finding, as ADR 0053 sets.
    DuplicateRow,
}

impl DropReason {
    /// Returns the text of the `reason` field of `dropped_rows`, for example `UnmappedRow`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnmappedRow => "UnmappedRow",
            Self::DuplicateRow => "DuplicateRow",
        }
    }
}

impl fmt::Display for DropReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The source rows of one filing that map to as-is facts, and the rows the parse dropped, by
/// reason.
///
/// # Examples
///
/// ```rust
/// use sec::shared::source_counts::{DropReason, SourceCounts};
///
/// let counts = SourceCounts::new(12, 0, &[]);
///
/// let expected_result = 0;
///
/// let result = counts.dropped_rows(DropReason::UnmappedRow);
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceCounts {
    rows_in: u64,
    dropped: BTreeMap<DropReason, u64>,
}

impl SourceCounts {
    /// Creates the counts of a filing with `rows_in` source rows, of which the parser merged
    /// `duplicate_rows` into an earlier row. Counts them under [`DropReason::DuplicateRow`], and
    /// one row under [`DropReason::UnmappedRow`] for each [`Finding::UnmappedRow`] in `findings`.
    #[must_use]
    pub fn new(rows_in: u64, duplicate_rows: u64, findings: &[Finding]) -> Self {
        let unmapped = findings
            .iter()
            .filter(|finding| matches!(finding, Finding::UnmappedRow { .. }))
            .count() as u64;
        let dropped = [
            (DropReason::UnmappedRow, unmapped),
            (DropReason::DuplicateRow, duplicate_rows),
        ]
        .into_iter()
        .filter(|&(_, rows)| rows > 0)
        .collect();
        Self { rows_in, dropped }
    }

    /// Returns the source rows of the filing.
    #[must_use]
    pub const fn rows_in(&self) -> u64 {
        self.rows_in
    }

    /// Returns the rows dropped for `reason`, or 0 if the parse dropped none for it.
    #[must_use]
    pub fn dropped_rows(&self, reason: DropReason) -> u64 {
        self.dropped.get(&reason).copied().unwrap_or(0)
    }

    /// Returns each reason with at least one dropped row, and its count, in the order of the
    /// reasons.
    #[must_use]
    pub const fn dropped(&self) -> &BTreeMap<DropReason, u64> {
        &self.dropped
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;
    use xbrl::core::concept::NamespaceFamily;
    use xbrl::core::filing::FilingId;
    use xbrl::core::invalid_value::{InvalidValue, InvalidValueReason};

    use super::*;

    fn filing_id() -> FilingId {
        FilingId::new("sec", "0000320193-26-000001").expect(
            "Given a hardcoded source and accession number, the creation should always succeed",
        )
    }

    fn unmapped_row() -> Finding {
        Finding::UnmappedRow {
            filing: filing_id(),
            row: "num.tsv Revenues us-gaap/2026 2026-09-30 1 USD 0x00000000 0".to_owned(),
            reason: InvalidValue::new(
                "NumRow",
                InvalidValueReason::NoMatchingRow("dim.tsv"),
                "0xff",
            ),
        }
    }

    fn undeclared_family() -> Finding {
        Finding::UndeclaredFamily {
            filing: filing_id(),
            family: NamespaceFamily::new("xyz/2027").expect(
                "Given a hardcoded family without whitespace, the creation should always succeed",
            ),
        }
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_source_counts() {
        implements_auto_traits::<SourceCounts>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_source_counts() {
        implements_value_traits::<SourceCounts>();
    }

    #[test]
    fn should_count_one_row_per_unmapped_row_finding_when_findings_mix_kinds() {
        let findings = vec![unmapped_row(), undeclared_family(), unmapped_row()];

        let expected_result = BTreeMap::from([(DropReason::UnmappedRow, 2)]);

        let result = SourceCounts::new(10, 0, &findings).dropped().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_no_reason_when_no_finding_is_an_unmapped_row() {
        let findings = vec![undeclared_family()];

        let expected_result = BTreeMap::new();

        let result = SourceCounts::new(10, 0, &findings).dropped().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_rows_in_when_created() {
        let rows_in = 27_529;

        let expected_result = 27_529;

        let result = SourceCounts::new(rows_in, 0, &[]).rows_in();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_as_dropped_rows_text_for_unmapped_row() {
        let reason = DropReason::UnmappedRow;

        let expected_result = "UnmappedRow";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_duplicate_rows_next_to_unmapped_rows_when_both_occur() {
        let findings = vec![unmapped_row()];

        let expected_result =
            BTreeMap::from([(DropReason::UnmappedRow, 1), (DropReason::DuplicateRow, 3)]);

        let result = SourceCounts::new(10, 3, &findings).dropped().clone();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_reason_as_dropped_rows_text_for_duplicate_row() {
        let reason = DropReason::DuplicateRow;

        let expected_result = "DuplicateRow";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }
}
