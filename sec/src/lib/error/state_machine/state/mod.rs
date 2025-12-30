//! # State Error Types
//!
//! This module defines error types that can occur within the internal logic of a state in the SEC state machine framework.
//! These errors represent failures related to input data, context, output computation, and state/context updates.
//!
//! ## Types
//! - [`State`]: Enum representing all error variants that can arise from state operations, including invalid CIK format, invalid input/context, failed output computation, and update failures.
//! - [`InvalidCikFormat`](invalid_cik_format): Error type for invalid CIK format, used as a variant in [`State`].
//!
//! ## Usage
//! Use [`State`] for error propagation and pattern matching when handling errors that originate from state logic. This enables granular error handling for state-specific failures within the broader state machine error hierarchy.
//!
//! ## Related Modules
//!
//! - [`crate::error`]: Top-level error types for the SEC state machine library, providing unified error handling across all components.
//! - [`crate::error::state_machine`]: Error types specific to state machine operations, such as transition failures and invalid state transitions.
//! - [`crate::error::state_machine::state::invalid_cik_format`]: Defines the [`InvalidCikFormat`] error type for handling invalid CIK format errors within state logic.
//! - [`crate::traits::state_machine::state`]: Core trait definitions for state behavior, including state lifecycle, data management, and error propagation.
//! - [`crate::traits::state_machine::state::state_data`]: Traits and types for managing and updating state data, including error handling for state data operations.
//! - [`crate::traits::state_machine::state::context_data`]: Traits and types for managing and updating context, including error handling for context operations.
//! - [`crate::traits::state_machine::transition`]: Traits for defining and handling state transitions, which may produce or propagate state errors.
//! - [`crate::traits::state_machine::super_state`]: Traits for hierarchical and composite states, which may aggregate or propagate errors from sub-states.
//! - [`crate::implementations::states`]: Concrete state implementations that use these error types for robust error handling in real-world scenarios.
//! - [`crate::implementations::states::extract::validate_cik_format`]: Example implementation that demonstrates how state errors, especially [`InvalidCikFormat`], are used in practice.
//!  
//! ## Example
//! ```rust
//! use sec::error::state_machine::state::{State, InvalidCikFormat};
//! let err = State::InvalidInput;
//! match err {
//!     State::InvalidCikFormat(invalid) => println!("Invalid CIK: {invalid}"),
//!     State::InvalidInput => println!("Input data is invalid"),
//!     State::InvalidContext => println!("Context data is invalid"),
//!     State::FailedOutputComputation => println!("Failed to compute output"),
//!     State::StateDataUpdateFailed => println!("Failed to update state data"),
//!     State::ContextUpdateFailed => println!("Failed to update context"),
//!      _ => println!("Other state error"),
//! }
//! ```

pub mod invalid_cik_format;
pub use invalid_cik_format::InvalidCikFormat;
pub mod invalid_sec_response;
pub use invalid_sec_response::InvalidSecResponse;
pub mod client_creation_failed;
pub use client_creation_failed::ClientCreationFailed;
pub mod request_execution_failed;
pub use request_execution_failed::RequestExecutionFailed;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum State {
    /// Invalid Cik format.
    InvalidCikFormat(InvalidCikFormat),

    /// Invalid SEC response.
    InvalidSecResponse(InvalidSecResponse),

    /// Indicates that the client creation has failed, which is typically due to an invalid configuration.
    ClientCreationFailed(ClientCreationFailed),

    /// Indicates that a SEC request execution has failed.
    RequestExecutionFailed(RequestExecutionFailed),

    /// Indicates that input data of a `State` is invalid and cannot be used to compute the output data.
    InvalidInput,

    /// Indicates that context of a `State` is invalid and cannot be used to compute the output data.
    InvalidContext,

    /// Indicates that the output computation of a `State` has failed.
    FailedOutputComputation,

    /// Indicates a failure during the update of the internal `StateData` of the `State`.
    StateDataUpdateFailed,

    /// Indicates a failure during the update of the `ContextData` of the `State`.
    ContextUpdateFailed,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem occured during internal state operations.")
    }
}

impl std::error::Error for State {}

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

    // #[test]
    // #[should_panic]
    // fn should_not_be_able_to_create_state_invalidcikformat_error_when_passing_valid_cik_string() {
    //     // TODO: "InvalidCikFormat error should only be able to be created passing an invalid CIK format, it should not be able to be created by passing a valid CIK format.");
    //     let _result = State::InvalidCikFormat("123456789".to_string());
    // }
}
