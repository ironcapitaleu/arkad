//! # Backend Error
//!
//! Provides [`BackendError`], the innermost leaf of the write-side error hierarchy: the
//! backend-level failures shared by every operation class. It embeds into
//! [`WriteError`] (and, later, `ReadError`) so the retryability decision lives
//! in one place.
//!
//! [`BackendError::is_retryable`] is the single decision the driver needs — retry versus
//! dead-letter — and only [`BackendError::Unavailable`] is transient.
//!
//! ## Usage
//!
//! ```rust
//! use storage::BackendError;
//!
//! let transient = BackendError::Unavailable { reason: "connection reset".to_string() };
//! assert!(transient.is_retryable());
//! ```

use thiserror::Error;

use super::ErrorKind;
use super::write_error::WriteError;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// A backend-level failure, shared across every operation class.
///
/// The innermost error in the hierarchy: it is embedded in the operation-classed errors
/// ([`WriteError`], and later `ReadError`) rather than surfaced on its own, and it is where
/// retryability is classified. Value type — rich backend detail is flattened to a `reason` string
/// at the conversion boundary.
pub enum BackendError {
    /// The backend is temporarily unavailable — the operation is safe to retry (connection drop,
    /// serialization failure, timeout).
    #[error("[Unavailable] Storage backend is temporarily unavailable, Reason: '{reason}'")]
    Unavailable {
        /// Human-readable explanation of why the backend is unavailable.
        reason: String,
    },

    /// The backend operation failed for a non-transient reason — retrying will not help.
    #[error("[Failed] Storage backend operation failed, Reason: '{reason}'")]
    Failed {
        /// Human-readable explanation of the failure.
        reason: String,
    },
}

impl BackendError {
    /// Returns `true` when the failure is transient and the operation may be retried.
    ///
    /// Only [`BackendError::Unavailable`] is retryable; [`BackendError::Failed`] denotes a
    /// permanent failure that a retry cannot resolve.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable { .. })
    }
}

impl From<BackendError> for WriteError {
    /// Upcasts a [`BackendError`] into a [`WriteError::Backend`] — the one-level, `?`-able upcast.
    fn from(error: BackendError) -> Self {
        Self::Backend(error)
    }
}

impl From<BackendError> for ErrorKind {
    /// Upcasts a [`BackendError`] straight to the top [`ErrorKind`] (skip-level), mirroring the
    /// skip-level [`TryFrom`] downcast.
    fn from(error: BackendError) -> Self {
        Self::Write(WriteError::Backend(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_backend_error() {
        implements_auto_traits::<BackendError>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_have_implemented_send_when_using_backend_error() {
        implements_send::<BackendError>();
    }

    #[test]
    const fn should_implement_sync_when_using_backend_error() {
        implements_sync::<BackendError>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_backend_error() {
        implements_send::<BackendError>();
        implements_sync::<BackendError>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_when_using_backend_error() {
        implements_sized::<BackendError>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_backend_error() {
        implements_hash::<BackendError>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_when_using_backend_error() {
        implements_partial_eq::<BackendError>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_backend_error() {
        implements_eq::<BackendError>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_when_using_backend_error() {
        implements_partial_ord::<BackendError>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_backend_error() {
        implements_ord::<BackendError>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_backend_error() {
        implements_debug::<BackendError>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_backend_error() {
        implements_clone::<BackendError>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_when_using_backend_error() {
        implements_unpin::<BackendError>();
    }

    #[test]
    fn should_be_retryable_when_backend_is_unavailable() {
        let error = BackendError::Unavailable {
            reason: "connection reset".to_string(),
        };

        let result = error.is_retryable();

        assert!(result);
    }

    #[test]
    fn should_not_be_retryable_when_backend_operation_failed() {
        let error = BackendError::Failed {
            reason: "constraint violation".to_string(),
        };

        let result = error.is_retryable();

        assert!(!result);
    }

    #[test]
    fn should_upcast_into_write_error_backend_variant_when_converting_from_backend_error() {
        let backend_error = BackendError::Failed {
            reason: "disk full".to_string(),
        };
        let expected_result = WriteError::Backend(backend_error.clone());

        let result: WriteError = backend_error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_upcast_into_error_kind_write_backend_when_converting_from_backend_error() {
        let backend_error = BackendError::Unavailable {
            reason: "timeout".to_string(),
        };
        let expected_result = ErrorKind::Write(WriteError::Backend(backend_error.clone()));

        let result: ErrorKind = backend_error.into();

        assert_eq!(result, expected_result);
    }
}
