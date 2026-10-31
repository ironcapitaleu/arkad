//! # Failed Document Write Error
//!
//! Provides the [`FailedDocumentWrite`] error: a domain-level [`FailedDocumentStorage`] enriched
//! with the name of the state in which the write failed.

use thiserror::Error;

use super::State as StateError;
use crate::shared::document_storage::FailedDocumentStorage;
use crate::traits::error::FromDomainError;

/// Error representing a rejected write to the store, tagged with the state it occurred in.
///
/// Wraps a domain-level [`FailedDocumentStorage`] together with the failing state's name, so a
/// storage failure carries state context as it propagates up the error hierarchy.
#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[FailedDocumentWrite] Failure in State: '{state_name}', Caused by: {domain_error}")]
pub struct FailedDocumentWrite {
    /// The name of the state where the error occurred.
    pub state_name: String,
    /// The domain-level error for the write the store rejected.
    #[source]
    pub domain_error: FailedDocumentStorage,
}

impl FailedDocumentWrite {
    /// Creates a new [`FailedDocumentWrite`] error.
    #[must_use]
    pub fn new(state_name: impl Into<String>, domain_error: FailedDocumentStorage) -> Self {
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

    /// Returns a reference to the underlying domain error.
    #[must_use]
    pub const fn domain_error(&self) -> &FailedDocumentStorage {
        &self.domain_error
    }
}

/// Converts a state-level [`FailedDocumentWrite`] error into the [`StateError`] enum variant.
impl From<FailedDocumentWrite> for StateError {
    fn from(domain_error: FailedDocumentWrite) -> Self {
        Self::FailedDocumentWrite(domain_error)
    }
}

/// Converts a domain-level [`FailedDocumentStorage`] into a state-level [`FailedDocumentWrite`]
/// error.
impl FromDomainError<FailedDocumentStorage> for FailedDocumentWrite {
    type DomainErr = FailedDocumentStorage;

    fn from_domain_error(state_name: impl Into<String>, err: Self::DomainErr) -> Self {
        Self::new(state_name, err)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::document_storage::DocumentStorageErrorReason;

    const STATE_NAME: &str = "Store Company Facts";

    fn sample_domain_error() -> FailedDocumentStorage {
        FailedDocumentStorage::new(DocumentStorageErrorReason::ConflictingWrite {
            details: "A document already exists under sec/companyfacts/CIK0001067983.json"
                .to_string(),
        })
    }

    #[test]
    fn should_return_state_name_when_state_name_method_is_called() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_domain_error());

        let expected_result = STATE_NAME;

        let result = error.state_name();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_domain_error_when_domain_error_method_is_called() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_domain_error());

        let expected_result = &sample_domain_error();

        let result = error.domain_error();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_to_state_error_when_from_is_called() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_domain_error());

        let expected_result = StateError::FailedDocumentWrite(error.clone());

        let result: StateError = error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_from_domain_error_when_from_domain_error_is_called() {
        let domain_error = sample_domain_error();

        let expected_result = FailedDocumentWrite::new(STATE_NAME, sample_domain_error());

        let result = FailedDocumentWrite::from_domain_error(STATE_NAME, domain_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_the_storage_error_after_caused_by_when_displayed() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_domain_error());

        let expected_result = "[FailedDocumentWrite] Failure in State: 'Store Company Facts', Caused by: [FailedDocumentStorage] Document storage failed, Caused by: [ConflictingWrite] Document conflicts with a stored document, Reason: 'A document already exists under sec/companyfacts/CIK0001067983.json'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_the_storage_error_as_source_when_wrapping_it() {
        let error = FailedDocumentWrite::new(STATE_NAME, sample_domain_error());

        let expected_result = Some(&sample_domain_error());

        let result = std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<FailedDocumentStorage>());

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
