//! # Retry
//!
//! Provides the run-level retry of the facade: [`with_retry`] runs a whole machine again after a
//! transient SEC failure, and [`DEFAULT_RETRY_DELAYS`] sets the waits between the attempts.
//! [`retry_while`] is the same loop for any error type.
//!
//! A rerun of a whole machine is safe, because a write with equal bytes changes nothing.

use std::future::Future;
use std::time::Duration;

use crate::error::{State as StateError, StateMachine};
use crate::prelude::StreamError;

/// The waits before the second and the third attempt of a machine that failed with a transient
/// SEC failure, without a call to `retry_delays`. After the third attempt the machine fails.
pub const DEFAULT_RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(10), Duration::from_mins(1)];

/// Runs `attempt` until it succeeds, fails with an error that is not transient, or has used every
/// wait of `retry_delays`. Calls `on_retry` with each error after which `attempt` runs again, and
/// returns the last error.
pub(super) async fn with_retry<T, F, Fut>(
    retry_delays: &[Duration],
    attempt: F,
    on_retry: impl Fn(&StreamError),
) -> Result<T, StreamError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, StreamError>>,
{
    retry_while(retry_delays, attempt, is_transient, on_retry).await
}

/// Runs `attempt` until it succeeds, fails with an error for which `is_transient` returns `false`,
/// or has used every wait of `retry_delays`. Calls `on_retry` with each error after which
/// `attempt` runs again, and returns the last error.
pub(super) async fn retry_while<T, E, F, Fut>(
    retry_delays: &[Duration],
    mut attempt: F,
    is_transient: impl Fn(&E) -> bool,
    on_retry: impl Fn(&E),
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut delays = retry_delays.iter();
    loop {
        match attempt().await {
            Ok(value) => return Ok(value),
            Err(error) => match delays.next() {
                Some(delay) if is_transient(&error) => {
                    on_retry(&error);
                    tokio::time::sleep(*delay).await;
                }
                _ => return Err(error),
            },
        }
    }
}

/// Returns `true` if a request of the machine failed with a transient SEC failure.
///
/// The transient failures are those of
/// [`FailedSecRequest::is_transient`](crate::shared::http_client::implementations::sec_client::error::FailedSecRequest::is_transient).
/// Every other state error, such as a folder without an XBRL instance, is permanent.
pub(super) fn is_transient(error: &StreamError) -> bool {
    matches!(
        &*error.source,
        StateMachine::State(StateError::FailedRequestExecution(failure))
            if failure.domain_error().is_transient()
    )
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use pretty_assertions::assert_eq;
    use serde_json::Value;
    use uuid::Uuid;

    use super::*;
    use crate::error::state_machine::state::{FailedDocumentSelection, FailedRequestExecution};
    use crate::prelude::StreamEvent;
    use crate::shared::filing_documents::{FilingDocumentRole, FilingDocumentsError};
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::http_client::implementations::sec_client::error::FailedSecRequest;
    use crate::shared::response::implementations::sec_response::error::{
        ErrorReason, InvalidSecResponse,
    };
    use crate::shared::status_code::StatusCode;

    fn stream_error(state_name: &str, source: StateMachine) -> StreamError {
        StreamError {
            event: StreamEvent::StateFailed,
            execution_id: Uuid::nil(),
            state_name: state_name.to_string(),
            data: Value::Null,
            source: Box::new(source),
        }
    }

    fn request_error(status_code: StatusCode) -> StreamError {
        let failure =
            FailedSecRequest::from(InvalidSecResponse::new(ErrorReason::InvalidStatusCode {
                status_code,
            }));
        stream_error(
            "Execute SEC Request",
            StateMachine::State(StateError::FailedRequestExecution(
                FailedRequestExecution::new("Execute SEC Request", failure),
            )),
        )
    }

    fn unavailable() -> StreamError {
        request_error(StatusCode::ServiceUnavailable)
    }

    fn not_found() -> StreamError {
        request_error(StatusCode::NotFound)
    }

    fn failed_selection(filing_documents_error: FilingDocumentsError) -> StreamError {
        stream_error(
            "Select Filing Documents",
            StateMachine::State(StateError::FailedDocumentSelection(
                FailedDocumentSelection::new("Select Filing Documents", filing_documents_error),
            )),
        )
    }

    fn accession_number() -> AccessionNumber {
        AccessionNumber::new("0000003545-26-000047").expect(
            "Given a hardcoded accession number of the right shape, creation should always succeed",
        )
    }

    /// Returns how many times `with_retry` runs an attempt that always fails with `error`.
    async fn attempts_until_given_up(error: fn() -> StreamError) -> usize {
        let attempts = AtomicUsize::new(0);
        let retry_delays = [Duration::ZERO, Duration::ZERO];
        let _ = with_retry(
            &retry_delays,
            || {
                attempts.fetch_add(1, Ordering::SeqCst);
                async move { Err::<(), _>(error()) }
            },
            |_| {},
        )
        .await;
        attempts.load(Ordering::SeqCst)
    }

    #[tokio::test]
    async fn should_try_once_more_per_retry_delay_when_every_attempt_gets_503() {
        let error = unavailable;

        let expected_result = 3;

        let result = attempts_until_given_up(error).await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_try_once_when_the_error_is_not_transient() {
        let error = not_found;

        let expected_result = 1;

        let result = attempts_until_given_up(error).await;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_error_as_transient_when_the_sec_answers_503() {
        let error = unavailable();

        let expected_result = true;

        let result = is_transient(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_error_as_permanent_when_the_sec_answers_404() {
        let error = not_found();

        let expected_result = false;

        let result = is_transient(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_error_as_permanent_when_the_folder_holds_no_instance() {
        let error = failed_selection(FilingDocumentsError::MissingXbrlInstance {
            accession_number: accession_number(),
        });

        let expected_result = false;

        let result = is_transient(&error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_count_an_error_as_permanent_when_the_folder_holds_two_instances() {
        let error = failed_selection(FilingDocumentsError::AmbiguousFilingDocument {
            accession_number: accession_number(),
            role: FilingDocumentRole::XbrlInstance,
            candidate_count: 2,
        });

        let expected_result = false;

        let result = is_transient(&error);

        assert_eq!(result, expected_result);
    }
}
