//! # Backend Error
//!
//! Provides [`BackendError`], the error raised when the storage backend cannot serve a request.
//!
//! ## Usage
//!
//! ```rust
//! use storage::BackendError;
//!
//! let reason = "connection reset";
//!
//! let _err = BackendError::unreachable_storage(reason);
//! ```

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::implementations::repository::filesystem::FailedFilesystemOperation;

#[non_exhaustive]
// `Display` and `Error` are written by hand, not derived with `thiserror`: `source()` returns the
// wrapped error only when the cause holds one, and `thiserror` cannot compute a source.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Error occurring at the storage backend.
///
/// Separates the different kinds of backend failures so a caller can tell them apart. Each kind
/// holds a [`BackendCause`]: a free-form reason, or the error it wraps.
pub enum BackendError {
    /// The backend cannot be reached (connection drop, timeout, host down).
    UnreachableStorage {
        /// Why the backend cannot be reached.
        cause: BackendCause,
    },

    /// The backend rejected or aborted the operation.
    FailedOperation {
        /// Why the operation failed.
        cause: BackendCause,
    },

    /// The backend refused the request because the caller lacks permission.
    UnauthorizedAccess {
        /// Why access was denied.
        cause: BackendCause,
    },
}

/// The cause of a [`BackendError`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum BackendCause {
    /// A free-form explanation. It ends the error chain and prints as `, Reason: '…'`.
    Reason(String),

    /// A filesystem error. It is the next link of the error chain and prints as
    /// `, Caused by: …`.
    FailedFilesystemOperation(FailedFilesystemOperation),
}

impl BackendError {
    /// Creates a [`BackendError::UnreachableStorage`] from the given reason.
    #[must_use]
    pub fn unreachable_storage(reason: impl Into<String>) -> Self {
        Self::UnreachableStorage {
            cause: BackendCause::Reason(reason.into()),
        }
    }

    /// Creates a [`BackendError::FailedOperation`] from the given reason.
    #[must_use]
    pub fn failed_operation(reason: impl Into<String>) -> Self {
        Self::FailedOperation {
            cause: BackendCause::Reason(reason.into()),
        }
    }

    /// Creates a [`BackendError::UnauthorizedAccess`] from the given reason.
    #[must_use]
    pub fn unauthorized_access(reason: impl Into<String>) -> Self {
        Self::UnauthorizedAccess {
            cause: BackendCause::Reason(reason.into()),
        }
    }

    /// Returns the cause of this error.
    #[must_use]
    pub const fn cause(&self) -> &BackendCause {
        match self {
            Self::UnreachableStorage { cause }
            | Self::FailedOperation { cause }
            | Self::UnauthorizedAccess { cause } => cause,
        }
    }
}

impl Display for BackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnreachableStorage { cause } => {
                write!(
                    f,
                    "[UnreachableStorage] Storage backend is not reachable{cause}"
                )
            }
            Self::FailedOperation { cause } => {
                write!(
                    f,
                    "[FailedOperation] Storage backend operation failed{cause}"
                )
            }
            Self::UnauthorizedAccess { cause } => {
                write!(
                    f,
                    "[UnauthorizedAccess] Storage backend denied access{cause}"
                )
            }
        }
    }
}

impl Error for BackendError {
    /// Returns the wrapped error, or `None` if the cause is a free-form reason.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.cause() {
            BackendCause::Reason(_) => None,
            BackendCause::FailedFilesystemOperation(error) => Some(error),
        }
    }
}

impl Display for BackendCause {
    /// Prints the segment that follows the description of a [`BackendError`].
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reason(reason) => write!(f, ", Reason: '{reason}'"),
            Self::FailedFilesystemOperation(error) => write!(f, ", Caused by: {error}"),
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
    const fn should_implement_auto_traits_for_backend_error() {
        implements_auto_traits::<BackendError>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_backend_error() {
        implements_send::<BackendError>();
    }

    #[test]
    const fn should_implement_sync_for_backend_error() {
        implements_sync::<BackendError>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_for_backend_error() {
        implements_sized::<BackendError>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_backend_error() {
        implements_hash::<BackendError>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_backend_error() {
        implements_partial_eq::<BackendError>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_backend_error() {
        implements_eq::<BackendError>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_backend_error() {
        implements_partial_ord::<BackendError>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_backend_error() {
        implements_ord::<BackendError>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_backend_error() {
        implements_debug::<BackendError>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_backend_error() {
        implements_clone::<BackendError>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_backend_error() {
        implements_unpin::<BackendError>();
    }

    #[test]
    fn should_build_unreachable_storage_variant_when_using_its_constructor() {
        let reason = "connection reset";

        let expected_result = BackendError::UnreachableStorage {
            cause: BackendCause::Reason(reason.to_owned()),
        };

        let result = BackendError::unreachable_storage(reason);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_failed_operation_variant_when_using_its_constructor() {
        let reason = "constraint violation";

        let expected_result = BackendError::FailedOperation {
            cause: BackendCause::Reason(reason.to_owned()),
        };

        let result = BackendError::failed_operation(reason);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_build_unauthorized_access_variant_when_using_its_constructor() {
        let reason = "role lacks INSERT";

        let expected_result = BackendError::UnauthorizedAccess {
            cause: BackendCause::Reason(reason.to_owned()),
        };

        let result = BackendError::unauthorized_access(reason);

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
