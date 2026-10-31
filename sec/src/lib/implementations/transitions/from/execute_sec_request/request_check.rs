//! # Request Check
//!
//! Provides [`ensure_request_matches_input`], which the store transitions call before they build
//! the storage key from the context of an
//! [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest).

use crate::error::state_machine::transition;
use crate::error::state_machine::transition::Transition as TransitionError;
use crate::shared::http_client::RespondsWith;
use crate::shared::request::SecRequest as SecRequestTrait;
use crate::shared::request::implementations::sec_request::SecRequest;

/// Checks that `request` is the request that `request_input` builds.
///
/// The store transitions key the stored document by `request_input`, so a mismatch stores a
/// response under the wrong key.
///
/// # Errors
///
/// Returns [`TransitionError::FailedContextConversion`] from
/// [`E::EXECUTE_STATE_NAME`](RespondsWith::EXECUTE_STATE_NAME) to `target_state_name` if the two
/// requests differ. The reason names the requested URL, then `input_label` and `request_input`,
/// for example `period '2024q1'`.
pub(super) fn ensure_request_matches_input<E: RespondsWith>(
    request: &SecRequest<E>,
    request_input: &E::Input,
    input_label: &str,
    target_state_name: &str,
) -> Result<(), TransitionError> {
    if *request == SecRequest::new(request_input.clone()) {
        return Ok(());
    }
    let reason = format!(
        "The request targets '{}', but the context names {input_label} '{request_input}'",
        request.inner.url(),
    );
    Err(
        transition::FailedContextConversion::new(E::EXECUTE_STATE_NAME, target_state_name, reason)
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

        let expected_result = Ok(());

        let result =
            ensure_request_matches_input(&request, &period("2024q1"), "period", TARGET_STATE_NAME);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_failed_context_conversion_when_the_request_input_builds_another_request() {
        let request = SecRequest::<NotesFile>::new(period("2023q4"));

        let expected_result = Err(TransitionError::FailedContextConversion(
            transition::FailedContextConversion::new(
                NotesFile::EXECUTE_STATE_NAME,
                TARGET_STATE_NAME,
                "The request targets 'https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2023q4_notes.zip', but the context names period '2024q1'",
            ),
        ));

        let result =
            ensure_request_matches_input(&request, &period("2024q1"), "period", TARGET_STATE_NAME);

        assert_eq!(result, expected_result);
    }
}
