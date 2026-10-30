//! # State Errors
//!
//! Provides the [`State`] error covering failures inside a state's own logic: CIK validation,
//! Notes period validation, index date validation, SEC request execution, a missing daily index,
//! document writes, data completeness, and input/context/output handling.
//!
//! The richer variants ([`InvalidCikFormat`], [`InvalidNotesPeriod`], [`InvalidIndexDate`], [`FailedRequestExecution`],
//! [`FailedDocumentWrite`], [`IncompleteCompanyFacts`]) wrap a domain error with the failing state's name.
//! [`MissingDailyIndex`] carries the failing state's name and the date. The rest are plain markers.
//! [`State`] is wrapped by [`StateMachine`](super::StateMachine) for propagation.
//!
//! ## Modules
//!
//! - [`invalid_cik_format`]: The [`InvalidCikFormat`] error wrapping a CIK validation failure.
//! - [`invalid_notes_period`]: The [`InvalidNotesPeriod`] error wrapping a Notes period validation failure.
//! - [`invalid_index_date`]: The [`InvalidIndexDate`] error wrapping an index date validation failure.
//! - [`failed_request_execution`]: The [`FailedRequestExecution`] error wrapping a failed SEC request.
//! - [`missing_daily_index`]: The [`MissingDailyIndex`] error for a daily index the SEC did not send.
//! - [`failed_document_write`]: The [`FailedDocumentWrite`] error wrapping a rejected store write.
//! - [`incomplete_company_facts`]: The [`IncompleteCompanyFacts`] error for a response missing required fields.

use thiserror::Error;

pub mod failed_document_write;
pub use failed_document_write::FailedDocumentWrite;
pub mod failed_request_execution;
pub use failed_request_execution::FailedRequestExecution;
pub mod incomplete_company_facts;
pub use incomplete_company_facts::IncompleteCompanyFacts;
pub mod invalid_cik_format;
pub use invalid_cik_format::InvalidCikFormat;
pub mod invalid_notes_period;
pub use invalid_notes_period::InvalidNotesPeriod;
pub mod invalid_index_date;
pub use invalid_index_date::InvalidIndexDate;
pub mod missing_daily_index;
pub use missing_daily_index::MissingDailyIndex;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Error occurring inside a state's own logic.
///
/// Spans the ways a state can fail while computing its output: a wrapped domain error (invalid
/// CIK, invalid Notes period, invalid index date, failed request, rejected write, incomplete facts) or a plain marker for invalid input, context, or a
/// failed update/computation.
pub enum State {
    /// Invalid Cik format.
    #[error("[StateError] A state level error occurred, Caused by: {0}")]
    InvalidCikFormat(#[source] InvalidCikFormat),

    /// Invalid Notes period.
    #[error("[StateError] A state level error occurred, Caused by: {0}")]
    InvalidNotesPeriod(#[source] InvalidNotesPeriod),

    /// Invalid index date.
    #[error("[StateError] A state level error occurred, Caused by: {0}")]
    InvalidIndexDate(#[source] InvalidIndexDate),

    /// Indicates that an SEC request execution has failed.
    #[error("[StateError] A state level error occurred, Caused by: {0}")]
    FailedRequestExecution(#[source] FailedRequestExecution),

    /// Indicates that the SEC sent no daily index of a date, and no quarter listing shows that
    /// the day had no filings.
    #[error("[StateError] A state level error occurred, Caused by: {0}")]
    MissingDailyIndex(#[source] MissingDailyIndex),

    /// Indicates that the store rejected the write of a document.
    #[error("[StateError] A state level error occurred, Caused by: {0}")]
    FailedDocumentWrite(#[source] FailedDocumentWrite),

    /// Indicates that the SEC Company Facts response is missing expected data fields.
    #[error("[StateError] A state level error occurred, Caused by: {0}")]
    IncompleteCompanyFacts(#[source] IncompleteCompanyFacts),

    /// Indicates that input data of a `State` is invalid and cannot be used to compute the output data.
    #[error(
        "[StateError] A state level error occurred, Reason: Invalid input data provided to state"
    )]
    InvalidInput,

    /// Indicates that context of a `State` is invalid and cannot be used to compute the output data.
    #[error(
        "[StateError] A state level error occurred, Reason: Invalid context data provided to state"
    )]
    InvalidContext,

    /// Indicates that the output computation of a `State` has failed.
    #[error("[StateError] A state level error occurred, Reason: Failed to compute output data")]
    FailedOutputComputation,

    /// Indicates a failure during the update of the internal `StateData` of the `State`.
    #[error("[StateError] A state level error occurred, Reason: Failed to update state data")]
    StateDataUpdateFailed,

    /// Indicates a failure during the update of the `ContextData` of the `State`.
    #[error("[StateError] A state level error occurred, Reason: Failed to update context data")]
    ContextUpdateFailed,
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_state_error() {
        implements_auto_traits::<State>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_state_error() {
        implements_send::<State>();
    }

    #[test]
    const fn should_implement_sync_for_state_error() {
        implements_sync::<State>();
    }

    #[test]
    const fn should_be_thread_safe_for_state_error() {
        implements_send::<State>();
        implements_sync::<State>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_for_state_error() {
        implements_sized::<State>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_state_error() {
        implements_hash::<State>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_state_error() {
        implements_partial_eq::<State>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_state_error() {
        implements_eq::<State>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_state_error() {
        implements_partial_ord::<State>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_state_error() {
        implements_ord::<State>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_state_error() {
        implements_debug::<State>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_state_error() {
        implements_clone::<State>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_state_error() {
        implements_unpin::<State>();
    }

    #[test]
    fn should_be_able_to_create_state_invalidinput_error_when_using_enum_directly() {
        let expected_result = State::InvalidInput;

        let result = State::InvalidInput;

        assert_eq!(result, expected_result);
    }
}
