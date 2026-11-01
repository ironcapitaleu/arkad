//! # Summary
//!
//! Provides [`AllRunSummary`], the outcome of both steps of one backfill pipeline run.

use chrono::NaiveDate;
use serde_json::{Value, json};
use thiserror::Error;

use super::FailedCikRead;
use crate::implementations::extraction::notes::{MissingCutOverDate, NotesRunSummary};
use crate::implementations::extraction::submissions::SubmissionsRunSummary;

/// The read of the CIKs of one stored Notes file: the number of distinct CIKs, or the error.
#[derive(Debug)]
pub(super) struct CikRead {
    /// The period of the Notes file, for example `2024q1`.
    pub(super) period: String,
    /// The number of distinct CIKs in the file, or the error of the read.
    pub(super) result: Result<usize, FailedCikRead>,
}

/// Error indicating a backfill pipeline run in which a Notes period, the CIK read of a stored
/// Notes file, or a `submissions` CIK failed.
#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "[FailedAllBackfill] Failed: {failed_periods} of {periods} Notes periods, {unreadable_files} of {stored_files} stored Notes files, {failed_ciks} of {ciks} submissions CIKs"
)]
pub struct FailedAllBackfill {
    /// The number of Notes periods that failed. An unpublished period does not count.
    pub failed_periods: usize,
    /// The number of Notes periods in the run.
    pub periods: usize,
    /// The number of stored Notes files whose CIKs the run could not read.
    pub unreadable_files: usize,
    /// The number of stored Notes files.
    pub stored_files: usize,
    /// The number of CIKs whose `submissions` file the run could not store.
    pub failed_ciks: usize,
    /// The number of distinct CIKs in the readable Notes files.
    pub ciks: usize,
}

/// The outcome of both steps of one backfill pipeline run.
///
/// The Notes step gives a [`NotesRunSummary`]. The `submissions` step gives the read of each
/// stored Notes file and a [`SubmissionsRunSummary`].
#[derive(Debug)]
pub struct AllRunSummary {
    pub(super) notes: NotesRunSummary,
    pub(super) cik_reads: Vec<CikRead>,
    pub(super) submissions: SubmissionsRunSummary,
}

impl AllRunSummary {
    /// Returns the summary of the Notes step.
    #[must_use]
    pub const fn notes(&self) -> &NotesRunSummary {
        &self.notes
    }

    /// Returns the summary of the `submissions` CIKs.
    #[must_use]
    pub const fn submissions(&self) -> &SubmissionsRunSummary {
        &self.submissions
    }

    /// Returns `Ok` if no Notes period, no CIK read, and no `submissions` CIK failed.
    ///
    /// # Errors
    ///
    /// Returns [`FailedAllBackfill`] if at least one of them failed.
    pub fn result(&self) -> Result<(), FailedAllBackfill> {
        let failed_periods = self.notes.failure_count();
        let unreadable_files = self.unreadable_file_count();
        let failed_ciks = self.submissions.failure_count();
        if failed_periods + unreadable_files + failed_ciks == 0 {
            return Ok(());
        }
        Err(FailedAllBackfill {
            failed_periods,
            periods: self.notes.success_count() + failed_periods + self.notes.unpublished_count(),
            unreadable_files,
            stored_files: self.cik_reads.len(),
            failed_ciks,
            ciks: self.submissions.success_count() + failed_ciks,
        })
    }

    /// Returns the cut-over date of the Notes step, as [`NotesRunSummary::cut_over_date`] does.
    ///
    /// The `submissions` step does not change it.
    ///
    /// # Errors
    ///
    /// Returns the [`MissingCutOverDate`] of the Notes step.
    pub fn cut_over_date(&self) -> Result<NaiveDate, MissingCutOverDate> {
        self.notes.cut_over_date()
    }

    /// Returns the number of stored Notes files whose CIKs the run could not read.
    #[must_use]
    pub fn unreadable_file_count(&self) -> usize {
        self.cik_reads
            .iter()
            .filter(|read| read.result.is_err())
            .count()
    }

    /// Returns one JSON entry per stored Notes file, in the order of the periods.
    ///
    /// A read file lists its number of distinct CIKs. An unreadable file lists the error.
    #[must_use]
    pub fn cik_read_entries(&self) -> Vec<Value> {
        self.cik_reads
            .iter()
            .map(|read| match &read.result {
                Ok(ciks) => json!({ "period": read.period, "ciks": ciks }),
                Err(error) => json!({ "period": read.period, "error": error.to_string() }),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_format_the_counts_when_displaying_failed_all_backfill() {
        let error = FailedAllBackfill {
            failed_periods: 0,
            periods: 2,
            unreadable_files: 1,
            stored_files: 2,
            failed_ciks: 3,
            ciks: 6587,
        };

        let expected_result = "[FailedAllBackfill] Failed: 0 of 2 Notes periods, 1 of 2 stored Notes files, 3 of 6587 submissions CIKs";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
