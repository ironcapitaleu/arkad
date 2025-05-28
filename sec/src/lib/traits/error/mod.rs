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
