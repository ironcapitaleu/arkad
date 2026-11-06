//! # Document Storage Errors
//!
//! Provides the [`FailedDocumentStorage`] error for a document that the store did not keep, and its
//! [`DocumentStorageErrorReason`].
//!
//! A storage error becomes an `sec` domain error at the call that returns it, so no `sec` error
//! type holds a `storage` type.

use std::fmt;

use storage::WriteError;
use thiserror::Error;

/// Error indicating that the store did not keep a document.
///
/// Wraps the [`DocumentStorageErrorReason`] that tells a rejected write from a failed backend.
///
/// Passes through: prints the inner error as it is, because this layer adds no fact.
/// The [`DocumentStorageErrorReason`] is no error type, so `source()` returns `None`.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("{reason}")]
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
        /// The reason the store gave for the conflict.
        details: String,
    },
    /// The write breaks an integrity rule of the store.
    FailedIntegrityCheck {
        /// The reason the store gave for the broken rule.
        details: String,
    },
    /// The storage backend failed or could not be reached.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    /// It holds the backend error as text, because no `sec` error type holds a `storage` type. So
    /// `source()` cannot reach the backend error.
    FailedBackend {
        /// The message of the backend error.
        details: String,
    },
    /// The store failed in a way that no other variant names.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    /// It holds the store error as text, so `source()` cannot reach the store error.
    Other {
        /// The message of the store error.
        details: String,
    },
}

impl fmt::Display for DocumentStorageErrorReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConflictingWrite { details } => write!(
                f,
                "[ConflictingWrite] Document conflicts with a stored document, Reason: '{details}'"
            ),
            Self::FailedIntegrityCheck { details } => write!(
                f,
                "[FailedIntegrityCheck] Document breaks an integrity rule of the store, Reason: '{details}'"
            ),
            Self::FailedBackend { details } | Self::Other { details } => f.write_str(details),
        }
    }
}

/// Converts a [`WriteError`] from the store into a [`FailedDocumentStorage`] error.
///
/// A rejected write keeps only the reason the store gave. A backend error keeps the message of
/// its `BackendError`. A `WriteError` variant with no match here becomes
/// [`DocumentStorageErrorReason::Other`] and keeps the full `WriteError` message.
impl From<WriteError> for FailedDocumentStorage {
    fn from(error: WriteError) -> Self {
        let reason = match error {
            WriteError::ConflictingWrite { reason } => {
                DocumentStorageErrorReason::ConflictingWrite { details: reason }
            }
            WriteError::FailedIntegrityCheck { reason } => {
                DocumentStorageErrorReason::FailedIntegrityCheck { details: reason }
            }
            WriteError::Backend(backend) => DocumentStorageErrorReason::FailedBackend {
                details: backend.to_string(),
            },
            other => DocumentStorageErrorReason::Other {
                details: other.to_string(),
            },
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

    const CONFLICT_REASON: &str =
        "A document already exists under sec/companyfacts/CIK0001067983.json";

    #[test]
    fn should_keep_the_reason_of_a_conflicting_write_when_converting_a_write_error() {
        let error = WriteError::conflicting_write(CONFLICT_REASON);

        let expected_result =
            FailedDocumentStorage::new(DocumentStorageErrorReason::ConflictingWrite {
                details: CONFLICT_REASON.to_string(),
            });

        let result = FailedDocumentStorage::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_reason_of_a_failed_integrity_check_when_converting_a_write_error() {
        let error = WriteError::failed_integrity_check("size does not match");

        let expected_result =
            FailedDocumentStorage::new(DocumentStorageErrorReason::FailedIntegrityCheck {
                details: "size does not match".to_string(),
            });

        let result = FailedDocumentStorage::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_backend_message_when_converting_a_backend_write_error() {
        let error = WriteError::from(BackendError::failed_operation("disk full"));

        let expected_result =
            FailedDocumentStorage::new(DocumentStorageErrorReason::FailedBackend {
                details: "[FailedOperation] Storage backend operation failed, Reason: 'disk full'"
                    .to_string(),
            });

        let result = FailedDocumentStorage::from(error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_reason_as_it_is_when_displayed() {
        let error = FailedDocumentStorage::new(DocumentStorageErrorReason::ConflictingWrite {
            details: CONFLICT_REASON.to_string(),
        });

        let expected_result = "[ConflictingWrite] Document conflicts with a stored document, Reason: 'A document already exists under sec/companyfacts/CIK0001067983.json'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_show_the_reason_after_its_name_for_conflicting_write() {
        let reason = DocumentStorageErrorReason::ConflictingWrite {
            details: CONFLICT_REASON.to_string(),
        };

        let expected_result = "[ConflictingWrite] Document conflicts with a stored document, Reason: 'A document already exists under sec/companyfacts/CIK0001067983.json'";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_show_the_reason_after_its_name_for_failed_integrity_check() {
        let reason = DocumentStorageErrorReason::FailedIntegrityCheck {
            details: "size does not match".to_string(),
        };

        let expected_result = "[FailedIntegrityCheck] Document breaks an integrity rule of the store, Reason: 'size does not match'";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_backend_message_as_it_is_for_failed_backend() {
        let reason = DocumentStorageErrorReason::FailedBackend {
            details: "[FailedOperation] Storage backend operation failed, Reason: 'disk full'"
                .to_string(),
        };

        let expected_result =
            "[FailedOperation] Storage backend operation failed, Reason: 'disk full'";

        let result = reason.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_store_message_as_it_is_for_other() {
        let reason = DocumentStorageErrorReason::Other {
            details: "[StoreClosed] Store is closed".to_string(),
        };

        let expected_result = "[StoreClosed] Store is closed";

        let result = reason.to_string();

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
