//! # Write Error
//!
//! Provides [`WriteError`], the operation-classed error every write method returns. Because a write
//! method returns a [`WriteError`] and a read method (a later slice) returns its own read error,
//! illegal states are unrepresentable — a `persist` can never hand back a read-only failure.
//!
//! [`WriteError::Backend`] wraps the shared [`BackendError`]; the two marker variants
//! ([`WriteError::ConflictingWrite`], [`WriteError::FailedIntegrityCheck`]) carry a flattened
//! `reason`. [`WriteError`] is wrapped by [`ErrorKind`] for the shared consumers (such as a retry
//! decorator) and recovers its [`BackendError`] via [`TryFrom`].
//!
//! ## Usage
//!
//! ```rust
//! use storage::WriteError;
//!
//! let _err = WriteError::FailedIntegrityCheck {
//!     reason: "SFAC-6 identity violated".to_string(),
//! };
//! ```

use thiserror::Error;

use super::ErrorKind;
use super::backend_error::BackendError;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Error representing a failure raised by a write operation.
///
/// The class returned directly by every write method. Value type — the marker variants flatten
/// their detail to a `reason` string, and [`WriteError::Backend`] embeds the shared
/// [`BackendError`] so backend failures keep their retryability classification.
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

    /// The write failed at the backend level; carries the shared [`BackendError`].
    #[error("[Backend] Storage backend error occurred, Caused by: {0}")]
    Backend(#[source] BackendError),
}

impl From<WriteError> for ErrorKind {
    /// Upcasts a [`WriteError`] into the top-level [`ErrorKind::Write`] — infallible and `?`-able.
    fn from(error: WriteError) -> Self {
        Self::Write(error)
    }
}

impl TryFrom<WriteError> for BackendError {
    type Error = ErrorKind;

    /// Extracts the [`BackendError`] from a [`WriteError::Backend`] — the fallible downcast.
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
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{fmt::Debug, hash::Hash};

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_write_error() {
        implements_auto_traits::<WriteError>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_have_implemented_send_when_using_write_error() {
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
    fn should_upcast_into_error_kind_write_when_converting_from_write_error() {
        let write_error = WriteError::ConflictingWrite {
            reason: "duplicate accession".to_string(),
        };
        let expected_result = ErrorKind::Write(write_error.clone());

        let result: ErrorKind = write_error.into();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_downcast_to_backend_error_when_write_error_is_a_backend_variant() {
        let backend_error = BackendError::Unavailable {
            reason: "timeout".to_string(),
        };
        let write_error = WriteError::Backend(backend_error.clone());

        let result = BackendError::try_from(write_error)
            .expect("Given a `WriteError::Backend`, the downcast to `BackendError` should succeed");

        assert_eq!(result, backend_error);
    }

    #[test]
    fn should_fail_downcast_to_backend_error_when_write_error_is_not_a_backend_variant() {
        let write_error = WriteError::ConflictingWrite {
            reason: "duplicate accession".to_string(),
        };
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = BackendError::try_from(write_error)
            .expect_err("A non-backend `WriteError` must not downcast into a `BackendError`");

        assert_eq!(result, expected_result);
    }
}
