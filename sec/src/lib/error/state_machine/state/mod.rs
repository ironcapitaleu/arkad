//! # State Errors
//!
//! Provides the [`State`] error covering failures inside a state's own logic: CIK validation, SEC
//! request execution, data completeness, and input/context/output handling.
//!
//! The richer variants ([`InvalidCikFormat`], [`FailedRequestExecution`],
//! [`IncompleteCompanyFacts`]) wrap a domain error with the failing state's name; the rest are
//! plain markers. [`State`] is wrapped by [`StateMachine`](super::StateMachine) for propagation.
//!
//! ## Modules
//!
//! - [`invalid_cik_format`]: The [`InvalidCikFormat`] error wrapping a CIK validation failure.
//! - [`failed_request_execution`]: The [`FailedRequestExecution`] error wrapping a failed SEC request.
//! - [`incomplete_company_facts`]: The [`IncompleteCompanyFacts`] error for a response missing required fields.

use thiserror::Error;

pub mod failed_request_execution;
pub use failed_request_execution::FailedRequestExecution;
pub mod incomplete_company_facts;
pub use incomplete_company_facts::IncompleteCompanyFacts;
pub mod invalid_cik_format;
pub use invalid_cik_format::InvalidCikFormat;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// An error occurring inside a state's own logic.
///
/// Spans the ways a state can fail while computing its output: a wrapped domain error (invalid
/// CIK, failed request, incomplete facts) or a plain marker for invalid input, context, or a
/// failed update/computation.
pub enum State {
    /// Invalid Cik format.
    #[error("[StateError] A state level error occurred, Caused by: {0}")]
    InvalidCikFormat(#[source] InvalidCikFormat),

    /// Indicates that a SEC request execution has failed.
    #[error("[StateError] A state level error occurred, Caused by: {0}")]
    FailedRequestExecution(#[source] FailedRequestExecution),

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
    use super::*;
    use std::{fmt::Debug, hash::Hash};
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_state() {
        implements_auto_traits::<State>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_have_implementend_send_when_using_state() {
        implements_send::<State>();
    }

    #[test]
    const fn should_implement_sync_when_using_state() {
        implements_sync::<State>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_state() {
        implements_send::<State>();
        implements_sync::<State>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_when_using_state() {
        implements_sized::<State>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_state() {
        implements_hash::<State>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_when_using_state() {
        implements_partial_eq::<State>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_state() {
        implements_eq::<State>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_when_using_state() {
        implements_partial_ord::<State>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_state() {
        implements_ord::<State>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_state() {
        implements_debug::<State>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_state() {
        implements_clone::<State>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_when_using_state() {
        implements_unpin::<State>();
    }

    #[test]
    fn should_be_able_to_create_state_invalidinputdata_error_when_using_enum_directly() {
        let _result = State::InvalidInput;
    }
}
