//! # Backend Error
//!
//! Provides [`BackendError`], the error raised when the storage backend cannot serve a request.
//!
//! ## Usage
//!
//! ```rust
//! use storage::BackendError;
//!
//! let _err = BackendError::unreachable_storage("connection reset");
//! ```

use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Error occurring at the storage backend.
///
/// Raised when the storage backend cannot serve a request. Detail arrives as a `reason` string
/// rather than a boxed source, which keeps the error a plain value.
pub enum BackendError {
    /// The backend cannot be reached (connection drop, timeout, host down).
    #[error("[UnreachableStorage] Storage backend is not reachable, Reason: '{reason}'")]
    UnreachableStorage {
        /// Human-readable explanation of why the backend cannot be reached.
        reason: String,
    },

    /// The backend rejected or aborted the operation.
    #[error("[FailedOperation] Storage backend operation failed, Reason: '{reason}'")]
    FailedOperation {
        /// Human-readable explanation of the failure.
        reason: String,
    },

    /// The backend refused the request because the caller lacks permission.
    #[error("[UnauthorizedAccess] Storage backend denied access, Reason: '{reason}'")]
    UnauthorizedAccess {
        /// Human-readable explanation of why access was denied.
        reason: String,
    },
}

impl BackendError {
    /// Creates a [`BackendError::UnreachableStorage`] from the given reason.
    #[must_use]
    pub fn unreachable_storage(reason: impl Into<String>) -> Self {
        Self::UnreachableStorage {
            reason: reason.into(),
        }
    }

    /// Creates a [`BackendError::FailedOperation`] from the given reason.
    #[must_use]
    pub fn failed_operation(reason: impl Into<String>) -> Self {
        Self::FailedOperation {
            reason: reason.into(),
        }
    }

    /// Creates a [`BackendError::UnauthorizedAccess`] from the given reason.
    #[must_use]
    pub fn unauthorized_access(reason: impl Into<String>) -> Self {
        Self::UnauthorizedAccess {
            reason: reason.into(),
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
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_backend_error() {
        implements_auto_traits::<BackendError>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_backend_error() {
        implements_send::<BackendError>();
    }

    #[test]
    const fn should_implement_sync_when_using_backend_error() {
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
    fn should_build_unreachable_storage_variant_when_using_its_constructor() {
        let expected_result = BackendError::UnreachableStorage {
            reason: "connection reset".to_owned(),
        };

        let result = BackendError::unreachable_storage("connection reset");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_failed_operation_variant_when_using_its_constructor() {
        let expected_result = BackendError::FailedOperation {
            reason: "constraint violation".to_owned(),
        };

        let result = BackendError::failed_operation("constraint violation");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_unauthorized_access_variant_when_using_its_constructor() {
        let expected_result = BackendError::UnauthorizedAccess {
            reason: "role lacks INSERT".to_owned(),
        };

        let result = BackendError::unauthorized_access("role lacks INSERT");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_reason_when_storage_is_unreachable() {
        let error = BackendError::unreachable_storage("connection reset");

        let expected_result =
            "[UnreachableStorage] Storage backend is not reachable, Reason: 'connection reset'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_reason_when_backend_operation_failed() {
        let error = BackendError::failed_operation("constraint violation");

        let expected_result =
            "[FailedOperation] Storage backend operation failed, Reason: 'constraint violation'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_and_reason_when_access_is_denied() {
        let error = BackendError::unauthorized_access("role lacks INSERT");

        let expected_result =
            "[UnauthorizedAccess] Storage backend denied access, Reason: 'role lacks INSERT'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
