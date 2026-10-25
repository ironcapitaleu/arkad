//! # Failed Document Write Error
//!
//! Provides the [`FailedDocumentWrite`] error: a [`WriteError`] from the store enriched with the
//! name of the state in which the write failed.

use storage::WriteError;
use thiserror::Error;

use super::State as StateError;
use crate::traits::error::FromDomainError;

/// Error representing a rejected write to the store, tagged with the state it occurred in.
///
/// Wraps the [`WriteError`] that the repository returned together with the failing state's name,
/// so a storage failure carries state context as it propagates up the error hierarchy.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[FailedDocumentWrite] Failure in State: '{state_name}', Caused by: {domain_error}")]
pub struct FailedDocumentWrite {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The error the repository returned for the write.
    #[source]
    pub domain_error: WriteError,
}

impl FailedDocumentWrite {
    /// Creates a new [`FailedDocumentWrite`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, domain_error: WriteError) -> Self {
        Self {
            state_name: state_name.into(),
            domain_error,
        }
    }

    /// Returns the name of the state where the error occurred.
    #[must_use]
    pub fn state_name(&self) -> &str {
        &self.state_name
    }

    /// Returns a reference to the error the repository returned.
    #[must_use]
    pub const fn domain_error(&self) -> &WriteError {
        &self.domain_error
    }
}

/// Converts a state-level [`FailedDocumentWrite`] error into the [`StateError`] enum variant.
impl From<FailedDocumentWrite> for StateError {
    fn from(domain_error: FailedDocumentWrite) -> Self {
        Self::FailedDocumentWrite(domain_error)
    }
}

/// Converts a [`WriteError`] into a state-level [`FailedDocumentWrite`] error.
impl FromDomainError<WriteError> for FailedDocumentWrite {
    type DomainErr = WriteError;

    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    const STATE_NAME: &str = "Store Company Facts";

    fn sample_write_error() -> WriteError {
        WriteError::conflicting_write("stored bytes differ")
    }

    #[test]
    fn should_return_state_name_when_state_name_method_is_called() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_write_error());

        let expected_result = STATE_NAME;

        let result = error.state_name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_domain_error_when_domain_error_method_is_called() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_write_error());

        let expected_result = &sample_write_error();

        let result = error.domain_error();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_write_error());

        let expected_result = StateError::FailedDocumentWrite(error.clone());

        let result: StateError = error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_from_domain_error_when_from_domain_error_is_called() {
        let expected_result = FailedDocumentWrite::new(STATE_NAME, sample_write_error());

        let result = FailedDocumentWrite::from_domain_error(STATE_NAME, sample_write_error());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_the_write_error_after_caused_by_when_displayed() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_write_error());

        let expected_result = "[FailedDocumentWrite] Failure in State: 'Store Company Facts', Caused by: [ConflictingWrite] Write conflicts with existing data, Reason: 'stored bytes differ'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_the_write_error_as_source_when_wrapping_it() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_write_error());

        let expected_result = Some(&sample_write_error());

        let result = std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<WriteError>());

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_failed_document_write() {
        implements_auto_traits::<FailedDocumentWrite>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_failed_document_write() {
        implements_value_traits::<FailedDocumentWrite>();
    }
}
