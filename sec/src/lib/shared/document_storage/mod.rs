//! # Document Storage Errors
//!
//! Provides the [`FailedDocumentStorage`] error for a document that the store did not keep, and its
//! [`DocumentStorageErrorReason`].
//!
//! A storage error becomes a `sec` domain error at the call that returns it, so no `sec` error
//! type holds a `storage` type.

use storage::WriteError;
use thiserror::Error;

/// Error indicating that the store did not keep a document.
///
/// Wraps the [`DocumentStorageErrorReason`] that tells a rejected write from a failed backend.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[FailedDocumentStorage] Document storage failed, Reason: '{details}'", details = .reason.details())]
pub struct FailedDocumentStorage {
    /// The reason why the store did not keep the document.
    pub reason: DocumentStorageErrorReason,
}

impl FailedDocumentStorage {
    /// Creates a new [`FailedDocumentStorage`] error.
    #[must_use]
    pub const fn new(reason: DocumentStorageErrorReason) -> Self {
        Self { reason }
    }
}

/// Enum representing the reason why the store did not keep a document.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocumentStorageErrorReason {
    /// The write conflicts with a document the store already holds.
    ConflictingWrite {
        /// A human-readable description of the conflict.
        details: String,
    },
    /// The write breaks an integrity rule of the store.
    FailedIntegrityCheck {
        /// A human-readable description of the broken rule.
        details: String,
    },
    /// The storage backend failed or could not be reached.
    FailedBackend {
        /// A human-readable description of the backend failure.
        details: String,
    },
    /// The store failed in a way that no other variant names.
    Other {
        /// A human-readable description of the failure.
        details: String,
    },
}

impl DocumentStorageErrorReason {
    /// Returns the human-readable description that the variant holds.
    #[must_use]
    pub fn details(&self) -> &str {
        match self {
            Self::ConflictingWrite { details }
            | Self::FailedIntegrityCheck { details }
            | Self::FailedBackend { details }
            | Self::Other { details } => details,
        }
    }
}

/// Converts a [`WriteError`] from the store into a [`FailedDocumentStorage`] error.
///
/// The details hold the `WriteError` message. A `WriteError` variant with no match here becomes
/// [`DocumentStorageErrorReason::Other`].
impl From<WriteError> for FailedDocumentStorage {
    fn from(error: WriteError) -> Self {
        let details = error.to_string();
        let reason = match error {
            WriteError::ConflictingWrite { .. } => {
                DocumentStorageErrorReason::ConflictingWrite { details }
            }
            WriteError::FailedIntegrityCheck { .. } => {
                DocumentStorageErrorReason::FailedIntegrityCheck { details }
            }
            WriteError::Backend(_) => DocumentStorageErrorReason::FailedBackend { details },
            _ => DocumentStorageErrorReason::Other { details },
        };
        Self::new(reason)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;
    use storage::BackendError;

    use super::*;

    #[test]
    fn should_map_a_conflicting_write_when_converting_a_write_error() {
        let error = WriteError::conflicting_write("stored bytes differ");

        let expected_result =
            FailedDocumentStorage::new(DocumentStorageErrorReason::ConflictingWrite {
                details: "[ConflictingWrite] Write conflicts with existing data, Reason: 'stored bytes differ'".to_string(),
            });

        let result = FailedDocumentStorage::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_map_a_failed_integrity_check_when_converting_a_write_error() {
        let error = WriteError::failed_integrity_check("size does not match");

        let expected_result =
            FailedDocumentStorage::new(DocumentStorageErrorReason::FailedIntegrityCheck {
                details: "[FailedIntegrityCheck] Data integrity or invariant violated, Reason: 'size does not match'".to_string(),
            });

        let result = FailedDocumentStorage::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_map_a_backend_error_to_failed_backend_when_converting_a_write_error() {
        let error = WriteError::from(BackendError::failed_operation("disk full"));

        let expected_result =
            FailedDocumentStorage::new(DocumentStorageErrorReason::FailedBackend {
                details: "[BackendError] Storage backend error occurred, Caused by: [FailedOperation] Storage backend operation failed, Reason: 'disk full'".to_string(),
            });

        let result = FailedDocumentStorage::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_show_the_details_after_reason_when_displayed() {
        let error = FailedDocumentStorage::new(DocumentStorageErrorReason::Other {
            details: "store closed".to_string(),
        });

        let expected_result =
            "[FailedDocumentStorage] Document storage failed, Reason: 'store closed'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_failed_document_storage() {
        implements_auto_traits::<FailedDocumentStorage>();
    }

    #[test]
    const fn should_implement_auto_traits_for_document_storage_error_reason() {
        implements_auto_traits::<DocumentStorageErrorReason>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_failed_document_storage() {
        implements_value_traits::<FailedDocumentStorage>();
    }

    #[test]
    const fn should_implement_value_traits_for_document_storage_error_reason() {
        implements_value_traits::<DocumentStorageErrorReason>();
    }
}
