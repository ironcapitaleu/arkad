//! # Backend Error
//!
//! Provides [`BackendError`], the innermost leaf of the error hierarchy: the backend-level
//! failures shared by every operation class. It is embedded in the operation-classed errors (such
//! as [`WriteError`]) rather than surfaced on its own.
//!
//! ## Usage
//!
//! ```rust
//! use storage::BackendError;
//!
//! let _err = BackendError::unavailable("connection reset");
//! ```

use thiserror::Error;

use super::ErrorKind;
use super::write_error::WriteError;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Error representing a backend-level failure, shared across every operation class.
///
/// The innermost error in the hierarchy: it is embedded in the operation-classed errors (such as
/// [`WriteError`]) rather than surfaced on its own. Value type — rich backend detail is flattened
/// to a `reason` string at the conversion boundary.
pub enum BackendError {
    /// The backend is temporarily unavailable (connection drop, serialization failure, timeout).
    #[error("[Unavailable] Storage backend is temporarily unavailable, Reason: '{reason}'")]
    Unavailable {
        /// Human-readable explanation of why the backend is unavailable.
        reason: String,
    },

    /// The backend operation failed for a non-transient reason.
    #[error("[Failed] Storage backend operation failed, Reason: '{reason}'")]
    Failed {
        /// Human-readable explanation of the failure.
        reason: String,
    },
}

impl BackendError {
    /// Creates a [`BackendError::Unavailable`] from the given reason.
    #[must_use]
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::Unavailable {
            reason: reason.into(),
        }
    }

    /// Creates a [`BackendError::Failed`] from the given reason.
    #[must_use]
    pub fn failed(reason: impl Into<String>) -> Self {
        Self::Failed {
            reason: reason.into(),
        }
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
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;

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
    fn should_upcast_into_write_error_backend_variant_when_converting_from_backend_error() {
        let backend_error = BackendError::failed("disk full");
        let expected_result = WriteError::Backend(backend_error.clone());

        let result: WriteError = backend_error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_upcast_into_error_kind_write_backend_when_converting_from_backend_error() {
        let backend_error = BackendError::unavailable("timeout");
        let expected_result = ErrorKind::Write(WriteError::Backend(backend_error.clone()));

        let result: ErrorKind = backend_error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_reason_when_backend_is_unavailable() {
        let error = BackendError::unavailable("connection reset");

        let expected_result =
            "[Unavailable] Storage backend is temporarily unavailable, Reason: 'connection reset'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_reason_when_backend_operation_failed() {
        let error = BackendError::failed("constraint violation");

        let expected_result =
            "[Failed] Storage backend operation failed, Reason: 'constraint violation'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
