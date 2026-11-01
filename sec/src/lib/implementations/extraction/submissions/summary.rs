//! # Summary
//!
//! Provides [`SubmissionsRunSummary`], the outcome of every CIK of one `submissions` batch run.

use serde_json::{Value, json};
use thiserror::Error;

use crate::prelude::StreamError;

/// Error indicating a `submissions` run in which at least one CIK failed.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("[FailedSubmissionsBackfill] {failures} of {ciks} submissions CIKs failed")]
pub struct FailedSubmissionsBackfill {
    /// The number of CIKs that failed.
    pub failures: usize,
    /// The number of CIKs in the run.
    pub ciks: usize,
}

/// The outcome of every CIK of one `submissions` batch run, in the order of the given CIKs.
///
/// A CIK either stored its `submissions` file or failed with the error of the state that failed.
#[derive(Debug)]
pub struct SubmissionsRunSummary(pub(super) Vec<(String, Result<(), StreamError>)>);

impl SubmissionsRunSummary {
    /// Returns `Ok` if every CIK stored its `submissions` file.
    ///
    /// # Errors
    ///
    /// Returns [`FailedSubmissionsBackfill`] if at least one CIK failed.
    pub fn result(&self) -> Result<(), FailedSubmissionsBackfill> {
        match self.failure_count() {
            0 => Ok(()),
            failures => Err(FailedSubmissionsBackfill {
                failures,
                ciks: self.0.len(),
            }),
        }
    }

    /// Returns the number of CIKs that stored their `submissions` file.
    #[must_use]
    pub fn success_count(&self) -> usize {
        self.0.iter().filter(|(_, outcome)| outcome.is_ok()).count()
    }

    /// Returns the number of CIKs that failed.
    #[must_use]
    pub fn failure_count(&self) -> usize {
        self.0.len() - self.success_count()
    }

    /// Returns one JSON entry per CIK.
    ///
    /// A stored CIK has the status `stored`. A failed CIK lists the failed state and the error.
    #[must_use]
    pub fn entries(&self) -> Vec<Value> {
        self.0
            .iter()
            .map(|(cik, outcome)| match outcome {
                Ok(()) => json!({ "cik": cik, "status": "stored" }),
                Err(error) => failed_entry(cik, error),
            })
            .collect()
    }

    /// Returns one JSON entry per failed CIK, with the failed state and the error.
    #[must_use]
    pub fn failed_entries(&self) -> Vec<Value> {
        self.0
            .iter()
            .filter_map(|(cik, outcome)| {
                outcome.as_ref().err().map(|error| failed_entry(cik, error))
            })
            .collect()
    }
}

/// Returns the JSON entry of the CIK `cik` that failed with `error`.
fn failed_entry(cik: &str, error: &StreamError) -> Value {
    json!({
        "cik": cik,
        "failed_state": error.state_name,
        "error": error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::error::StateMachine;
    use crate::prelude::StreamEvent;

    fn stored(cik: &str) -> (String, Result<(), StreamError>) {
        (cik.to_string(), Ok(()))
    }

    fn failed(cik: &str) -> (String, Result<(), StreamError>) {
        let error = StreamError {
            event: StreamEvent::StateFailed,
            execution_id: uuid::Uuid::nil(),
            state_name: "Validate CIK Format".to_string(),
            data: Value::Null,
            source: StateMachine::InvalidConfiguration,
        };
        (cik.to_string(), Err(error))
    }

    #[test]
    fn should_return_ok_when_every_cik_stored_its_submissions_file() {
        let summary = SubmissionsRunSummary(vec![stored("320193"), stored("1067983")]);

        let expected_result = Ok(());

        let result = summary.result();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_the_stored_and_the_failed_ciks_when_the_run_mixes_outcomes() {
        let summary =
            SubmissionsRunSummary(vec![failed("ABC"), stored("320193"), stored("1067983")]);

        let expected_result = (2, 1);

        let result = (summary.success_count(), summary.failure_count());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_the_status_or_the_error_of_each_cik_when_listing_entries() {
        let summary = SubmissionsRunSummary(vec![failed("ABC"), stored("320193")]);

        let expected_result = vec![
            json!({
                "cik": "ABC",
                "failed_state": "Validate CIK Format",
                "error": "[StreamError][00000000-0000-0000-0000-000000000000] 'state_failed' in 'Validate CIK Format', Caused by: Invalid configuration of the state machine",
            }),
            json!({ "cik": "320193", "status": "stored" }),
        ];

        let result = summary.entries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_list_only_the_failed_ciks_when_listing_failed_entries() {
        let summary =
            SubmissionsRunSummary(vec![stored("320193"), failed("ABC"), stored("1067983")]);

        let expected_result = vec![json!({
            "cik": "ABC",
            "failed_state": "Validate CIK Format",
            "error": "[StreamError][00000000-0000-0000-0000-000000000000] 'state_failed' in 'Validate CIK Format', Caused by: Invalid configuration of the state machine",
        })];

        let result = summary.failed_entries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_the_counts_when_displaying_failed_submissions_backfill() {
        let error = FailedSubmissionsBackfill {
            failures: 2,
            ciks: 6587,
        };

        let expected_result = "[FailedSubmissionsBackfill] 2 of 6587 submissions CIKs failed";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
