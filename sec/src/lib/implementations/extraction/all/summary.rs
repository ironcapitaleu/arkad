//! # Summary
//!
//! Provides [`AllRunSummary`], the outcome of both steps of one backfill pipeline run.

use std::time::Duration;

use chrono::NaiveDate;
use thiserror::Error;

use crate::implementations::extraction::daily_filings::DailyRunSummary;
use crate::implementations::extraction::notes::NotesRunSummary;

/// Error indicating a backfill pipeline run in which a Notes period or a part of the catch-up
/// failed.
#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "[FailedAllBackfill] Failed: {failed_periods} of {periods} Notes periods, {catch_up_failures} dates, filings, or submissions files in {catch_up_dates} catch-up dates"
)]
pub struct FailedAllBackfill {
    /// The number of Notes periods that failed. An unpublished period counts only if the `HEAD`
    /// request found the newest period and the run stored no period that ends on or after the
    /// cut-over date.
    pub failed_periods: usize,
    /// The number of Notes periods in the run.
    pub periods: usize,
    /// The number of failed dates, filings, and `submissions` files of the catch-up.
    pub catch_up_failures: usize,
    /// The number of dates of the catch-up.
    pub catch_up_dates: usize,
}

/// The outcome of both steps of one backfill pipeline run.
///
/// The Notes step gives a [`NotesRunSummary`]. The catch-up gives a [`DailyRunSummary`].
#[derive(Debug)]
pub struct AllRunSummary {
    pub(super) notes: NotesRunSummary,
    pub(super) notes_duration: Duration,
    pub(super) cut_over_date: NaiveDate,
    pub(super) newest_published: bool,
    pub(super) catch_up_dates: Vec<NaiveDate>,
    pub(super) catch_up: DailyRunSummary,
    pub(super) catch_up_duration: Duration,
}

impl AllRunSummary {
    /// Returns the summary of the Notes step.
    #[must_use]
    pub const fn notes(&self) -> &NotesRunSummary {
        &self.notes
    }

    /// Returns the time that the Notes step took.
    #[must_use]
    pub const fn notes_duration(&self) -> Duration {
        self.notes_duration
    }

    /// Returns the cut-over date: the last day of the newest published Notes period. The catch-up
    /// starts [`CATCH_UP_LOOKBACK_DAYS`](super::CATCH_UP_LOOKBACK_DAYS) days before it.
    #[must_use]
    pub const fn cut_over_date(&self) -> NaiveDate {
        self.cut_over_date
    }

    /// Returns the dates of the catch-up, oldest first.
    #[must_use]
    pub fn catch_up_dates(&self) -> &[NaiveDate] {
        &self.catch_up_dates
    }

    /// Returns the summary of the catch-up.
    #[must_use]
    pub const fn catch_up(&self) -> &DailyRunSummary {
        &self.catch_up
    }

    /// Returns the time that the catch-up took.
    #[must_use]
    pub const fn catch_up_duration(&self) -> Duration {
        self.catch_up_duration
    }

    /// Returns `Ok` if no Notes period and no part of the catch-up failed.
    ///
    /// # Errors
    ///
    /// Returns [`FailedAllBackfill`] if at least one of them failed. A pending date and an
    /// unextractable filing do not count as failed. An unpublished Notes period does not count
    /// either, unless the `HEAD` request found the newest period and no stored period ends on or
    /// after the cut-over date. In that case the catch-up starts after the newest stored period,
    /// and the days between are in neither path.
    pub fn result(&self) -> Result<(), FailedAllBackfill> {
        let unpublished_periods = self.notes.unpublished_count();
        let failed_periods = if self.misses_cut_over_period() {
            self.notes.failure_count() + unpublished_periods
        } else {
            self.notes.failure_count()
        };
        let catch_up_failures = self
            .catch_up
            .result()
            .err()
            .map_or(0, |failed| failed.failures);
        if failed_periods + catch_up_failures == 0 {
            return Ok(());
        }
        Err(FailedAllBackfill {
            failed_periods,
            periods: self.notes.success_count() + self.notes.failure_count() + unpublished_periods,
            catch_up_failures,
            catch_up_dates: self.catch_up_dates.len(),
        })
    }

    /// Returns `true` if the `HEAD` request found the newest period, and no stored Notes period
    /// ends on or after the cut-over date.
    fn misses_cut_over_period(&self) -> bool {
        self.newest_published
            && self
                .notes
                .stored_archives()
                .map(|archive| archive.period().last_day())
                .max()
                .is_none_or(|newest| newest < self.cut_over_date)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_format_the_counts_when_displaying_failed_all_backfill() {
        let error = FailedAllBackfill {
            failed_periods: 1,
            periods: 3,
            catch_up_failures: 2,
            catch_up_dates: 14,
        };

        let expected_result = "[FailedAllBackfill] Failed: 1 of 3 Notes periods, 2 dates, filings, or submissions files in 14 catch-up dates";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
