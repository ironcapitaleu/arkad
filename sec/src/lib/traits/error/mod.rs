//! # Error Traits
//!
//! Provides the [`FromDomainError`] trait for lifting a domain-level error into a state-level one.
//!
//! Domain errors (such as CIK validation failures) know nothing about which state they occurred
//! in. This trait is the single, uniform way to wrap such an error with that state context, so
//! every state-level error is built consistently and converts cleanly into
//! [`State`](crate::error::State).

use crate::error::State as StateError;
use std::error::Error;

/// Builds a state-level error from a domain error, tagging it with the failing state's name.
///
/// Implemented by the state-level error wrappers (e.g.
/// [`InvalidCikFormat`](crate::error::state_machine::state::InvalidCikFormat)). The
/// [`Into<StateError>`] bound guarantees the result slots into the [`State`](crate::error::State)
/// hierarchy.
///
/// # Associated Types
///
/// - `DomainErr`: The domain-level error type being wrapped.
pub trait FromDomainError<DomainErr>: Error + Into<StateError> + Sized {
    /// The domain-level error type being wrapped.
    type DomainErr: Error + 'static;

    /// Wraps a domain error with the name of the state in which it occurred.
    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self;
}
