//! # State Error Types
//!
//! This module defines error types that can occur within the internal logic of a state in the SEC state machine framework.
//! These errors represent failures related to CIK validation, SEC request execution, data validation, and state/context updates.
//!
//! ## Types
//! - [`State`]: Enum representing all error variants that can arise from state operations.
//! - [`InvalidCikFormat`]: Error type for invalid CIK format validation failures.
//! - [`FailedRequestExecution`]: Error type for failed SEC request execution.
//! - [`IncompleteCompanyFacts`]: Error type for incomplete or missing data in SEC Company Facts responses.
//!
//! ## Usage
//! Use [`State`] for error propagation and pattern matching when handling errors that originate from state logic. This enables granular error handling for state-specific failures within the broader state machine error hierarchy.
//!
//! ## Related Modules
//!
//! - [`crate::error`]: Top-level error types for the SEC state machine library, providing unified error handling across all components.
//! - [`crate::error::state_machine`]: Error types specific to state machine operations, such as transition failures and invalid state transitions.
//! - [`crate::traits::state_machine::state`]: Core trait definitions for state behavior, including state lifecycle, data management, and error propagation.
//! - [`crate::implementations::states`]: Concrete state implementations that use these error types for robust error handling in real-world scenarios.

use thiserror::Error;

pub mod failed_request_execution;
pub use failed_request_execution::FailedRequestExecution;
pub mod incomplete_company_facts;
pub use incomplete_company_facts::IncompleteCompanyFacts;
pub mod invalid_cik_format;
pub use invalid_cik_format::InvalidCikFormat;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
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
