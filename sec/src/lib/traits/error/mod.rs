//! # Error Handling Traits
//!
//! This module defines traits for error handling and domain error conversions within the SEC state machine library.
//! These traits enable consistent and extensible error propagation from domain-specific errors to state-level and state machine-level errors.
//!
//! ## Traits
//! - [`FromDomainError`]: Trait for converting domain-level errors into state-level errors, enriching them with state context.
//!
//! ## Usage
//! Implement these traits for your custom error types to enable seamless error conversion and propagation in state and state machine implementations.
//! This ensures that errors originating from domain logic (such as CIK validation) can be wrapped with additional context and handled uniformly by the state machine framework.
//!
//! ## See Also
//! - [`crate::error`]: Strongly-typed error definitions for states, transitions, and state machines.
//! - [`crate::traits::state_machine`]: Core traits for state machine extensibility and integration.

use crate::error::State as StateError;
use std::error::Error;

/// Trait for constructing a state-level error from a domain-level error,
/// enriching it with state context such as the state name.
///
/// # Type Parameters
/// - `DomainErr`: The domain-level error type to wrap.
pub trait FromDomainError<DomainErr>: Error + Into<StateError> + Sized {
    /// The domain-level error type to wrap.
    type DomainErr: Error + 'static;

    /// Constructs a state-level error from a domain error and state-level context.
    ///
    /// # Arguments
    /// * `state_name` - The name of the state where the error occurred.
    /// * `err` - The domain-level error to wrap.
    fn from_domain_error(state_name: &(impl ToString + ?Sized), err: Self::DomainErr) -> Self;
}
