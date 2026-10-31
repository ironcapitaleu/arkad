//! # Request Check
//!
//! Provides [`ensure_request_matches_input`], which the store transitions call before they build
//! the storage key from the context of an
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest).

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::implementations::states::extract::execute_sec_request::constants::STATE_NAME as EXECUTE_SEC_REQUEST;
use crate::shared::request::SecRequest as SecRequestTrait;
use crate::shared::request::implementations::sec_request::{RequestKind, SecRequest};

/// Checks that `request` is the request that `request_input` builds: the same input and the same
/// URL.
///
/// The store transitions key the stored document by `request_input`, so a mismatch stores a
/// response under the wrong key.
///
/// # Errors
///
/// Returns [`TransitionError::FailedContextConversion`] from [`EXECUTE_SEC_REQUEST`] to
/// `target_state_name` if the two requests differ. The reason names the requested URL, the input
/// of the request, and `request_input`, each input after `input_label`, for example
/// `period '2024q1'`.
pub(super) fn ensure_request_matches_input<K: RequestKind>(
    request: &SecRequest<K>,
    request_input: &K::Input,
    input_label: &str,
    target_state_name: &str,
) -> Result<(), TransitionError> {
    let expected_request = SecRequest::<K>::new(request_input.clone());
    if *request == expected_request && request.inner.url() == expected_request.inner.url() {
        return Ok(());
    }
    let reason = format!(
        "The request targets '{}' for {input_label} '{}', but the context names {input_label} '{request_input}'",
        request.inner.url(),
        request.input(),
    );
    Err(
        transition::FailedContextConversion::new(EXECUTE_SEC_REQUEST, target_state_name, reason)
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::notes_period::NotesPeriod;
    use crate::shared::request::implementations::sec_request::NotesFile;

    const TARGET_STATE_NAME: &str = "Store Notes Archive";

    fn period(raw_period: &str) -> NotesPeriod {
        NotesPeriod::new(raw_period)
            .expect("Given a hardcoded published period, parsing should always succeed")
    }

    #[test]
    fn should_pass_when_the_request_input_builds_the_request() {
        let request = SecRequest::<NotesFile>::new(period("2024q1"));
        let request_input = period("2024q1");
        let input_label = "period";

        let expected_result = Ok(());

        let result =
            ensure_request_matches_input(&request, &request_input, input_label, TARGET_STATE_NAME);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_context_conversion_when_the_request_input_builds_another_request() {
        let request = SecRequest::<NotesFile>::new(period("2023q4"));
        let request_input = period("2024q1");
        let input_label = "period";

        let expected_result = Err(TransitionError::FailedContextConversion(
            transition::FailedContextConversion::new(
                EXECUTE_SEC_REQUEST,
                TARGET_STATE_NAME,
                "The request targets 'https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2023q4_notes.zip' for period '2023q4', but the context names period '2024q1'",
            ),
        ));

        let result =
            ensure_request_matches_input(&request, &request_input, input_label, TARGET_STATE_NAME);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_context_conversion_when_the_request_url_differs_from_its_input() {
        let mut request = SecRequest::<NotesFile>::new(period("2024q1"));
        *request.inner.url_mut() = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2023q4_notes.zip"
            .parse()
            .expect("Given a hardcoded HTTPS URL literal, Url parsing should always succeed");
        let request_input = period("2024q1");
        let input_label = "period";

        let expected_result = Err(TransitionError::FailedContextConversion(
            transition::FailedContextConversion::new(
                EXECUTE_SEC_REQUEST,
                TARGET_STATE_NAME,
                "The request targets 'https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2023q4_notes.zip' for period '2024q1', but the context names period '2024q1'",
            ),
        ));

        let result =
            ensure_request_matches_input(&request, &request_input, input_label, TARGET_STATE_NAME);

        assert_eq!(result, expected_result);
    }
}
