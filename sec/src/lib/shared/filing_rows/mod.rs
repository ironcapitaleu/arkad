//! # Filing Rows
//!
//! Provides [`FilingRows`], the rows of one filing for the four datasets of a transform unit.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use state_maschine::prelude::StateData as SMStateData;
use xbrl::core::filing::FilingId;

use crate::datasets::{AsIsFactRow, CandidateRow, FilingRow, FindingRow};
use crate::error::State as StateError;
use crate::shared::source_counts::SourceCounts;
use crate::traits::state_machine::state::StateData;

/// The rows of one filing for `as_is_facts`, `filings`, `candidates`, and `findings`, and the
/// counts of its source rows.
#[derive(Debug, Clone)]
pub struct FilingRows {
    /// The filing of the rows.
    pub filing_id: FilingId,
    /// One row per as-is fact.
    pub as_is_facts: Vec<AsIsFactRow>,
    /// The row of the filing.
    pub filing: FilingRow,
    /// One row per candidate, the ones a conflict blocks included.
    pub candidates: Vec<CandidateRow>,
    /// The `UnmappedRow`, `UndeclaredFamily`, and `Conflict` findings, with `step = filing`.
    pub findings: Vec<FindingRow>,
    /// The source rows of the filing and the rows the parse dropped.
    pub counts: SourceCounts,
}

// Deviation: `CandidateRow` and `FindingRow` implement no `Ord`, and `CandidateRow` no `Hash`, but a
// state data type must implement both. So equality, hashing, and order use the filing ID and the
// provenance of the filing row only. The pair names the rows of one source of one filing. A test
// of a state compares the rows field by field, since `==` does not compare them.
impl PartialEq for FilingRows {
    fn eq(&self, other: &Self) -> bool {
        self.filing_id == other.filing_id && self.filing.provenance == other.filing.provenance
    }
}

impl Eq for FilingRows {}

impl Hash for FilingRows {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.filing_id.hash(state);
        self.filing.provenance.hash(state);
    }
}

impl PartialOrd for FilingRows {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FilingRows {
    fn cmp(&self, other: &Self) -> Ordering {
        self.filing_id
            .cmp(&other.filing_id)
            .then_with(|| self.filing.provenance.cmp(&other.filing.provenance))
    }
}

// Deviation: the update is `()`. No code changes a value of this type in place, so it needs no
// field-wise updater.
impl SMStateData for FilingRows {
    type UpdateType = ();

    fn state(&self) -> &Self {
        self
    }

    fn update_state(&mut self, (): Self::UpdateType) {}
}

impl StateData for FilingRows {
    /// Changes nothing: the update holds no field.
    fn update_state(&mut self, (): Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_filing_rows() {
        implements_auto_traits::<FilingRows>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_filing_rows() {
        implements_value_traits::<FilingRows>();
    }
}
