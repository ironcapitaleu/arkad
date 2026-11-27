//! # Errors
//!
//! Provides the error types the `storage` crate returns.
//!
//! Errors nest by operation: a [`ReadError`], a [`WriteError`], or a [`LeaseError`] wraps a
//! backend error, and an [`ErrorKind`] wraps that. A caller can therefore propagate one type and
//! still recover the specific cause. The [`TryFrom`] impls extract the inner error, returning
//! [`ErrorKind::DowncastNotPossible`] when the variant does not match.
//!
//! A [`BackendError`] reaches [`ErrorKind`] through the operation that raised it. The backend error
//! alone does not say which operation that was, so it has no direct conversion to [`ErrorKind`].
//!
//! ## Modules
//!
//! - [`backend_error`]: Errors raised by the storage backend.
//! - [`lease_error`]: Errors raised while taking or reading a lease.
//! - [`read_error`]: Errors raised while reading from the store.
//! - [`write_error`]: Errors raised while writing to the store.
//!
//! ## Usage
//!
//! ```rust
//! use storage::{ErrorKind, WriteError};
//!
//! let write_error = WriteError::conflicting_write("duplicate accession number");
//!
//! let _err = ErrorKind::Write(write_error);
//! ```

use thiserror::Error;

pub mod backend_error;
pub mod lease_error;
pub mod read_error;
pub mod write_error;

pub use backend_error::{BackendCause, BackendError};
pub use lease_error::LeaseError;
pub use read_error::ReadError;
pub use write_error::WriteError;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// The error type the `storage` crate returns.
///
/// Wraps the error of each operation, so a caller propagates one type. [`TryFrom`] extracts a
/// lower-level error from it, and returns [`ErrorKind::DowncastNotPossible`] when the variant does
/// not match.
pub enum ErrorKind {
    /// An error originating from a read operation.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    Read(ReadError),

    /// An error originating from a write operation.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    Write(WriteError),

    /// An error originating from a lease operation.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    Lease(LeaseError),

    /// A [`TryFrom`] downcast to a more specific error type did not match the held variant.
    #[error("[DowncastNotPossible] Failed to downcast error into a more specific type")]
    DowncastNotPossible,
}

impl From<ReadError> for ErrorKind {
    /// Converts a [`ReadError`] into the [`ErrorKind::Read`] variant.
    fn from(error: ReadError) -> Self {
        Self::Read(error)
    }
}

impl From<WriteError> for ErrorKind {
    /// Converts a [`WriteError`] into the [`ErrorKind::Write`] variant.
    fn from(error: WriteError) -> Self {
        Self::Write(error)
    }
}

impl From<LeaseError> for ErrorKind {
    /// Converts a [`LeaseError`] into the [`ErrorKind::Lease`] variant.
    fn from(error: LeaseError) -> Self {
        Self::Lease(error)
    }
}

impl TryFrom<ErrorKind> for LeaseError {
    type Error = ErrorKind;

    /// Extracts the [`LeaseError`] from an [`ErrorKind::Lease`].
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not an [`ErrorKind::Lease`].
    fn try_from(value: ErrorKind) -> Result<Self, Self::Error> {
        match value {
            ErrorKind::Lease(lease) => Ok(lease),
            _ => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

impl TryFrom<ErrorKind> for ReadError {
    type Error = ErrorKind;

    /// Extracts the [`ReadError`] from an [`ErrorKind::Read`].
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not an [`ErrorKind::Read`].
    fn try_from(value: ErrorKind) -> Result<Self, Self::Error> {
        match value {
            ErrorKind::Read(read) => Ok(read),
            _ => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

impl TryFrom<ErrorKind> for WriteError {
    type Error = ErrorKind;

    /// Extracts the [`WriteError`] from an [`ErrorKind::Write`].
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not an [`ErrorKind::Write`].
    fn try_from(value: ErrorKind) -> Result<Self, Self::Error> {
        match value {
            ErrorKind::Write(write) => Ok(write),
            _ => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

impl TryFrom<ErrorKind> for BackendError {
    type Error = ErrorKind;

    /// Extracts the [`BackendError`] from an [`ErrorKind`] holding a read, a write, or a lease
    /// operation that failed at the backend.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if no branch holds a [`BackendError`].
    fn try_from(value: ErrorKind) -> Result<Self, Self::Error> {
        match value {
            ErrorKind::Read(ReadError::Backend(backend))
            | ErrorKind::Write(WriteError::Backend(backend))
            | ErrorKind::Lease(LeaseError::Backend(backend)) => Ok(backend),
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
    const fn should_implement_auto_traits_for_error_kind() {
        implements_auto_traits::<ErrorKind>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_for_error_kind() {
        implements_send::<ErrorKind>();
    }

    #[test]
    const fn should_implement_sync_for_error_kind() {
        implements_sync::<ErrorKind>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_for_error_kind() {
        implements_sized::<ErrorKind>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_for_error_kind() {
        implements_hash::<ErrorKind>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_for_error_kind() {
        implements_partial_eq::<ErrorKind>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_for_error_kind() {
        implements_eq::<ErrorKind>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_for_error_kind() {
        implements_partial_ord::<ErrorKind>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_for_error_kind() {
        implements_ord::<ErrorKind>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_for_error_kind() {
        implements_debug::<ErrorKind>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_for_error_kind() {
        implements_clone::<ErrorKind>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_for_error_kind() {
        implements_unpin::<ErrorKind>();
    }

    #[test]
    fn should_wrap_read_error_into_read_variant_when_converting_from_read_error() {
        let read_error = ReadError::missing_record("0000320193-24-000123");
        let expected_result = ErrorKind::Read(read_error.clone());

        let result = ErrorKind::from(read_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_wrap_write_error_into_write_variant_when_converting_from_write_error() {
        let write_error = WriteError::conflicting_write("duplicate accession");
        let expected_result = ErrorKind::Write(write_error.clone());

        let result = ErrorKind::from(write_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_downcast_to_read_error_when_error_kind_is_a_read_variant() {
        let error_kind = ErrorKind::Read(ReadError::missing_record("0000320193-24-000123"));

        let expected_result = ReadError::MissingRecord {
            identifier: "0000320193-24-000123".to_owned(),
        };

        let result = ReadError::try_from(error_kind)
            .expect("Given an `ErrorKind::Read`, the downcast to `ReadError` should succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_downcast_to_read_error_when_error_kind_is_not_a_read_variant() {
        let error_kind = ErrorKind::Write(WriteError::conflicting_write("duplicate accession"));
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = ReadError::try_from(error_kind)
            .expect_err("A non-read `ErrorKind` should not downcast into a `ReadError`");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_downcast_to_write_error_when_error_kind_is_a_write_variant() {
        let error_kind = ErrorKind::Write(WriteError::conflicting_write("duplicate accession"));

        let expected_result = WriteError::ConflictingWrite {
            reason: "duplicate accession".to_owned(),
        };

        let result = WriteError::try_from(error_kind)
            .expect("Given an `ErrorKind::Write`, the downcast to `WriteError` should succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_downcast_to_write_error_when_error_kind_is_not_a_write_variant() {
        let error_kind = ErrorKind::Read(ReadError::missing_record("0000320193-24-000123"));
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = WriteError::try_from(error_kind)
            .expect_err("A non-write `ErrorKind` should not downcast into a `WriteError`");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_level_downcast_to_backend_error_when_error_kind_wraps_a_backend_read() {
        let error_kind = ErrorKind::Read(ReadError::Backend(BackendError::unreachable_storage(
            "timeout",
        )));

        let expected_result = BackendError::UnreachableStorage {
            cause: BackendCause::Reason("timeout".to_owned()),
        };

        let result = BackendError::try_from(error_kind).expect(
            "Given an `ErrorKind` wrapping a `ReadError::Backend`, the skip-level downcast should succeed",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_level_downcast_to_backend_error_when_error_kind_wraps_a_backend_write() {
        let error_kind = ErrorKind::Write(WriteError::Backend(BackendError::unreachable_storage(
            "timeout",
        )));

        let expected_result = BackendError::UnreachableStorage {
            cause: BackendCause::Reason("timeout".to_owned()),
        };

        let result = BackendError::try_from(error_kind).expect(
            "Given an `ErrorKind` wrapping a `WriteError::Backend`, the skip-level downcast should succeed",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_skip_level_downcast_to_backend_error_when_no_branch_holds_a_backend_error() {
        let error_kind = ErrorKind::Write(WriteError::failed_integrity_check(
            "balance sheet identity violated",
        ));
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = BackendError::try_from(error_kind).expect_err(
            "An `ErrorKind` holding no backend error should not skip-level downcast into a `BackendError`",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_roundtrip_lease_error_when_upcast_then_downcast() {
        let lease_error = LeaseError::Backend(BackendError::unreachable_storage("timeout"));

        let expected_result = LeaseError::Backend(BackendError::UnreachableStorage {
            cause: BackendCause::Reason("timeout".to_owned()),
        });

        let result = LeaseError::try_from(ErrorKind::from(lease_error))
            .expect("A `LeaseError` upcast into `ErrorKind` should downcast back unchanged");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_level_downcast_to_backend_error_when_error_kind_wraps_a_backend_lease() {
        let error_kind = ErrorKind::Lease(LeaseError::Backend(BackendError::unreachable_storage(
            "timeout",
        )));

        let expected_result = BackendError::UnreachableStorage {
            cause: BackendCause::Reason("timeout".to_owned()),
        };

        let result = BackendError::try_from(error_kind).expect(
            "Given an `ErrorKind` wrapping a `LeaseError::Backend`, the skip-level downcast should succeed",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_roundtrip_read_error_when_upcast_then_downcast() {
        let read_error = ReadError::missing_record("0000320193-24-000123");

        let expected_result = ReadError::MissingRecord {
            identifier: "0000320193-24-000123".to_owned(),
        };

        let result = ReadError::try_from(ErrorKind::from(read_error))
            .expect("A `ReadError` upcast into `ErrorKind` should downcast back unchanged");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_roundtrip_write_error_when_upcast_then_downcast() {
        let write_error = WriteError::failed_integrity_check("balance sheet identity violated");

        let expected_result = WriteError::FailedIntegrityCheck {
            reason: "balance sheet identity violated".to_owned(),
        };

        let result = WriteError::try_from(ErrorKind::from(write_error))
            .expect("A `WriteError` upcast into `ErrorKind` should downcast back unchanged");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_read_error_as_it_is_when_error_kind_wraps_read() {
        let error_kind = ErrorKind::Read(ReadError::missing_record("0000320193-24-000123"));

        let expected_result = "[MissingRecord] Requested record '0000320193-24-000123' not found";

        let result = error_kind.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_write_error_as_it_is_when_error_kind_wraps_write() {
        let error_kind = ErrorKind::Write(WriteError::conflicting_write("duplicate accession"));

        let expected_result =
            "[ConflictingWrite] Write conflicts with existing data, Reason: 'duplicate accession'";

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
    fn should_return_no_source_when_error_kind_wraps_read() {
        let error_kind = ErrorKind::Read(ReadError::missing_record("0000320193-24-000123"));

        let expected_result: Option<String> = None;

        let result = std::error::Error::source(&error_kind).map(ToString::to_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_source_when_error_kind_wraps_write() {
        let error_kind = ErrorKind::Write(WriteError::failed_integrity_check(
            "balance sheet identity violated",
        ));

        let expected_result: Option<String> = None;

        let result = std::error::Error::source(&error_kind).map(ToString::to_string);

        assert_eq!(result, expected_result);
    }
}
