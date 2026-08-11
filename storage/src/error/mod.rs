//! # Errors
//!
//! Provides the write-side error hierarchy for the `storage` crate, topped by [`ErrorKind`].
//!
//! The hierarchy is operation-classed and value-typed at every level: module-per-level, `From`
//! upcast / [`TryFrom`] downcast, and a single [`ErrorKind::DowncastNotPossible`] sentinel on the
//! top. Each class is keyed to the *kind of operation* — a write method returns a [`WriteError`],
//! so illegal states are unrepresentable — and each embeds the shared [`BackendError`]. Methods
//! return the narrow class; the union [`ErrorKind`] is what shared consumers take, reached by the
//! `From` upcast.
//!
//! Every level is `#[non_exhaustive]`, so introducing further operation classes or variants stays
//! additive rather than breaking.
//!
//! ## Modules
//!
//! - [`backend_error`]: The shared [`BackendError`] leaf.
//! - [`write_error`]: The [`WriteError`] operation class returned by every write method.
//!
//! ## Usage
//!
//! ```rust
//! use storage::{ErrorKind, WriteError};
//!
//! let _err = ErrorKind::Write(WriteError::ConflictingWrite {
//!     reason: "duplicate accession number".to_string(),
//! });
//! ```

pub mod backend_error;
pub mod write_error;

use thiserror::Error;

pub use backend_error::BackendError;
pub use write_error::WriteError;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// The top-level, operation-classed error type for the `storage` crate.
///
/// The outermost layer of the hierarchy: it wraps every more specific write error so callers can
/// propagate one type, and supports downward extraction to [`WriteError`] / [`BackendError`] via
/// [`TryFrom`].
pub enum ErrorKind {
    /// An error originating from a write operation.
    #[error("[Write] Problem occurred during a write operation, Caused by: {0}")]
    Write(#[source] WriteError),

    /// A [`TryFrom`] downcast to a more specific error type did not match the held variant.
    #[error("[DowncastNotPossible] Failed to downcast error into a more specific type")]
    DowncastNotPossible,
}

impl TryFrom<ErrorKind> for WriteError {
    type Error = ErrorKind;

    /// Extracts the [`WriteError`] from an [`ErrorKind::Write`] — the fallible downcast.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not an [`ErrorKind::Write`].
    fn try_from(value: ErrorKind) -> Result<Self, Self::Error> {
        match value {
            ErrorKind::Write(write) => Ok(write),
            ErrorKind::DowncastNotPossible => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

impl TryFrom<ErrorKind> for BackendError {
    type Error = ErrorKind;

    /// Extracts the [`BackendError`] from an [`ErrorKind`] wrapping a [`WriteError::Backend`] — the
    /// fallible skip-level downcast.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not an [`ErrorKind::Write`]
    /// wrapping a [`WriteError::Backend`] (skip-level downcast).
    fn try_from(value: ErrorKind) -> Result<Self, Self::Error> {
        match value {
            ErrorKind::Write(WriteError::Backend(backend)) => Ok(backend),
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
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_error_kind() {
        implements_auto_traits::<ErrorKind>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_have_implemented_send_when_using_error_kind() {
        implements_send::<ErrorKind>();
    }

    #[test]
    const fn should_implement_sync_when_using_error_kind() {
        implements_sync::<ErrorKind>();
    }

    #[test]
    const fn should_be_thread_safe_when_using_error_kind() {
        implements_send::<ErrorKind>();
        implements_sync::<ErrorKind>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_when_using_error_kind() {
        implements_sized::<ErrorKind>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_error_kind() {
        implements_hash::<ErrorKind>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_when_using_error_kind() {
        implements_partial_eq::<ErrorKind>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_error_kind() {
        implements_eq::<ErrorKind>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_when_using_error_kind() {
        implements_partial_ord::<ErrorKind>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_error_kind() {
        implements_ord::<ErrorKind>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_error_kind() {
        implements_debug::<ErrorKind>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_error_kind() {
        implements_clone::<ErrorKind>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_when_using_error_kind() {
        implements_unpin::<ErrorKind>();
    }

    #[test]
    fn should_downcast_to_write_error_when_error_kind_is_a_write_variant() {
        let write_error = WriteError::ConflictingWrite {
            reason: "duplicate accession".to_string(),
        };
        let error_kind = ErrorKind::Write(write_error.clone());

        let result = WriteError::try_from(error_kind)
            .expect("Given an `ErrorKind::Write`, the downcast to `WriteError` should succeed");

        assert_eq!(result, write_error);
    }

    #[test]
    fn should_fail_downcast_to_write_error_when_error_kind_is_not_a_write_variant() {
        let error_kind = ErrorKind::DowncastNotPossible;
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = WriteError::try_from(error_kind)
            .expect_err("A non-write `ErrorKind` must not downcast into a `WriteError`");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_level_downcast_to_backend_error_when_error_kind_wraps_a_backend_write() {
        let backend_error = BackendError::Unavailable {
            reason: "timeout".to_string(),
        };
        let error_kind = ErrorKind::Write(WriteError::Backend(backend_error.clone()));

        let result = BackendError::try_from(error_kind).expect(
            "Given an `ErrorKind` wrapping a `WriteError::Backend`, the skip-level downcast should succeed",
        );

        assert_eq!(result, backend_error);
    }

    #[test]
    fn should_fail_skip_level_downcast_to_backend_error_when_write_is_not_a_backend_variant() {
        let error_kind = ErrorKind::Write(WriteError::FailedIntegrityCheck {
            reason: "SFAC-6 identity violated".to_string(),
        });
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = BackendError::try_from(error_kind).expect_err(
            "A non-backend `WriteError` must not skip-level downcast into a `BackendError`",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_roundtrip_write_error_when_upcast_then_downcast() {
        let write_error = WriteError::FailedIntegrityCheck {
            reason: "SFAC-6 identity violated".to_string(),
        };

        let upcast: ErrorKind = write_error.clone().into();
        let result = WriteError::try_from(upcast)
            .expect("A `WriteError` upcast into `ErrorKind` should downcast back unchanged");

        assert_eq!(result, write_error);
    }

    #[test]
    fn should_chain_write_display_after_caused_by_when_error_kind_wraps_write() {
        let error_kind = ErrorKind::Write(WriteError::ConflictingWrite {
            reason: "duplicate accession".to_string(),
        });

        let expected_result = "[Write] Problem occurred during a write operation, Caused by: [ConflictingWrite] Write conflicts with existing data, Reason: 'duplicate accession'";

        let result = error_kind.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_when_downcast_is_not_possible() {
        let error_kind = ErrorKind::DowncastNotPossible;

        let expected_result =
            "[DowncastNotPossible] Failed to downcast error into a more specific type";

        let result = error_kind.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_write_error_as_source_when_error_kind_wraps_write() {
        let write_error = WriteError::FailedIntegrityCheck {
            reason: "SFAC-6 identity violated".to_string(),
        };
        let error_kind = ErrorKind::Write(write_error.clone());

        let expected_result = Some(&write_error);

        let result = std::error::Error::source(&error_kind)
            .and_then(|source| source.downcast_ref::<WriteError>());

        assert_eq!(result, expected_result);
    }
}
