//! # Write Error
//!
//! Provides [`WriteError`], the error raised when the store cannot complete a write.
//!
//! ## Usage
//!
//! ```rust
//! use storage::WriteError;
//!
//! let _err = WriteError::failed_integrity_check("SFAC-6 identity violated");
//! ```

use thiserror::Error;

use super::ErrorKind;
use super::backend_error::BackendError;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Error occurring while writing to the store.
///
/// Separates a conflict with existing data, a violated integrity invariant, and a failure at the
/// storage backend. The conflict and integrity variants flatten their detail into a `reason`
/// string, so the error stays a plain value.
pub enum WriteError {
    /// The write conflicts with data already present (unique violation, already-exists).
    #[error("[ConflictingWrite] Write conflicts with existing data, Reason: '{reason}'")]
    ConflictingWrite {
        /// Human-readable explanation of the conflict.
        reason: String,
    },

    /// The write violates a data-integrity invariant (check constraint, SFAC-6 identity).
    #[error("[FailedIntegrityCheck] Data integrity or invariant violated, Reason: '{reason}'")]
    FailedIntegrityCheck {
        /// Human-readable explanation of the violated invariant.
        reason: String,
    },

    /// The write failed at the storage backend.
    #[error("[Backend] Storage backend error occurred, Caused by: {0}")]
    Backend(#[source] BackendError),
}

impl WriteError {
    /// Creates a [`WriteError::ConflictingWrite`] from the given reason.
    #[must_use]
    pub fn conflicting_write(reason: impl Into<String>) -> Self {
        Self::ConflictingWrite {
            reason: reason.into(),
        }
    }

    /// Creates a [`WriteError::FailedIntegrityCheck`] from the given reason.
    #[must_use]
    pub fn failed_integrity_check(reason: impl Into<String>) -> Self {
        Self::FailedIntegrityCheck {
            reason: reason.into(),
        }
    }
}

impl From<BackendError> for WriteError {
    /// Converts a [`BackendError`] into a [`WriteError::Backend`] variant.
    fn from(error: BackendError) -> Self {
        Self::Backend(error)
    }
}

impl TryFrom<WriteError> for BackendError {
    type Error = ErrorKind;

    /// Extracts the [`BackendError`] from a [`WriteError::Backend`].
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not a [`WriteError::Backend`].
    fn try_from(value: WriteError) -> Result<Self, Self::Error> {
        match value {
            WriteError::Backend(backend) => Ok(backend),
            _ => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_write_error() {
        implements_auto_traits::<WriteError>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_write_error() {
        implements_send::<WriteError>();
    }

    #[test]
    const fn should_implement_sync_when_using_write_error() {
        implements_sync::<WriteError>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_write_error() {
        implements_send::<WriteError>();
        implements_sync::<WriteError>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_when_using_write_error() {
        implements_sized::<WriteError>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_write_error() {
        implements_hash::<WriteError>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_when_using_write_error() {
        implements_partial_eq::<WriteError>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_write_error() {
        implements_eq::<WriteError>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_when_using_write_error() {
        implements_partial_ord::<WriteError>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_write_error() {
        implements_ord::<WriteError>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_write_error() {
        implements_debug::<WriteError>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_write_error() {
        implements_clone::<WriteError>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_when_using_write_error() {
        implements_unpin::<WriteError>();
    }

    #[test]
    fn should_wrap_backend_error_into_backend_variant_when_converting_from_backend_error() {
        let backend_error = BackendError::failed("disk full");
        let expected_result = WriteError::Backend(backend_error.clone());

        let result = WriteError::from(backend_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_downcast_to_backend_error_when_write_error_is_a_backend_variant() {
        let backend_error = BackendError::unavailable("timeout");
        let write_error = WriteError::Backend(backend_error.clone());

        let result = BackendError::try_from(write_error)
            .expect("Given a `WriteError::Backend`, the downcast to `BackendError` must succeed");

        assert_eq!(result, backend_error);
    }

    #[test]
    fn should_fail_downcast_to_backend_error_when_write_error_is_not_a_backend_variant() {
        let write_error = WriteError::conflicting_write("duplicate accession");
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = BackendError::try_from(write_error)
            .expect_err("A non-backend `WriteError` must not downcast into a `BackendError`");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_reason_when_write_conflicts() {
        let error = WriteError::conflicting_write("duplicate accession");

        let expected_result =
            "[ConflictingWrite] Write conflicts with existing data, Reason: 'duplicate accession'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_reason_when_integrity_check_fails() {
        let error = WriteError::failed_integrity_check("SFAC-6 identity violated");

        let expected_result = "[FailedIntegrityCheck] Data integrity or invariant violated, Reason: 'SFAC-6 identity violated'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_backend_display_after_caused_by_when_write_error_wraps_backend() {
        let error = WriteError::Backend(BackendError::failed("disk full"));

        let expected_result = "[Backend] Storage backend error occurred, Caused by: [Failed] Storage backend operation failed, Reason: 'disk full'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_backend_error_as_source_when_write_error_wraps_backend() {
        let backend_error = BackendError::unavailable("timeout");
        let error = WriteError::Backend(backend_error.clone());

        let expected_result = Some(&backend_error);

        let result = std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<BackendError>());

        assert_eq!(result, expected_result);
    }
}
