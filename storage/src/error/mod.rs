//! # Errors
//!
//! Provides the error types the `storage` crate returns.
//!
//! Errors nest by operation: a [`ReadError`] or a [`WriteError`] wraps a backend error, and an
//! [`ErrorKind`] wraps that. A caller can therefore propagate one type and still recover the
//! specific cause. The [`TryFrom`] impls extract the inner error, returning
//! [`ErrorKind::DowncastNotPossible`] when the variant does not match.
//!
//! A [`BackendError`] reaches [`ErrorKind`] through the operation that raised it. The backend error
//! alone does not say which operation that was, so it has no direct conversion to [`ErrorKind`].
//!
//! ## Modules
//!
//! - [`backend_error`]: Errors raised by the storage backend.
//! - [`read_error`]: Errors raised while reading from the store.
//! - [`write_error`]: Errors raised while writing to the store.
//!
//! ## Usage
//!
//! ```rust
//! use storage::{ErrorKind, WriteError};
//!
//! let _err = ErrorKind::Write(WriteError::conflicting_write("duplicate accession number"));
//! ```

use thiserror::Error;

pub mod backend_error;
pub mod read_error;
pub mod write_error;

pub use backend_error::BackendError;
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
    #[error("[Read] Problem occurred during a read operation, Caused by: {0}")]
    Read(#[source] ReadError),

    /// An error originating from a write operation.
    #[error("[Write] Problem occurred during a write operation, Caused by: {0}")]
    Write(#[source] WriteError),

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

    /// Extracts the [`BackendError`] from an [`ErrorKind`] holding a read or a write that failed at
    /// the backend.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if neither the read nor the write branch holds a
    /// [`BackendError`].
    fn try_from(value: ErrorKind) -> Result<Self, Self::Error> {
        match value {
            ErrorKind::Read(ReadError::Backend(backend))
            | ErrorKind::Write(WriteError::Backend(backend)) => Ok(backend),
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
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_error_kind() {
        implements_auto_traits::<ErrorKind>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_error_kind() {
        implements_send::<ErrorKind>();
    }

    #[test]
    const fn should_implement_sync_when_using_error_kind() {
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
    fn should_wrap_read_error_into_read_variant_when_converting_from_read_error() {
        let read_error = ReadError::MissingRecord;
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
        let read_error = ReadError::MissingRecord;
        let error_kind = ErrorKind::Read(read_error.clone());

        let result = ReadError::try_from(error_kind)
            .expect("Given an `ErrorKind::Read`, the downcast to `ReadError` should succeed");

        assert_eq!(result, read_error);
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
        let write_error = WriteError::conflicting_write("duplicate accession");
        let error_kind = ErrorKind::Write(write_error.clone());

        let result = WriteError::try_from(error_kind)
            .expect("Given an `ErrorKind::Write`, the downcast to `WriteError` should succeed");

        assert_eq!(result, write_error);
    }

    #[test]
    fn should_fail_downcast_to_write_error_when_error_kind_is_not_a_write_variant() {
        let error_kind = ErrorKind::Read(ReadError::MissingRecord);
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = WriteError::try_from(error_kind)
            .expect_err("A non-write `ErrorKind` should not downcast into a `WriteError`");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_level_downcast_to_backend_error_when_error_kind_wraps_a_backend_read() {
        let backend_error = BackendError::unreachable_storage("timeout");
        let error_kind = ErrorKind::Read(ReadError::Backend(backend_error.clone()));

        let result = BackendError::try_from(error_kind).expect(
            "Given an `ErrorKind` wrapping a `ReadError::Backend`, the skip-level downcast should succeed",
        );

        assert_eq!(result, backend_error);
    }

    #[test]
    fn should_skip_level_downcast_to_backend_error_when_error_kind_wraps_a_backend_write() {
        let backend_error = BackendError::unreachable_storage("timeout");
        let error_kind = ErrorKind::Write(WriteError::Backend(backend_error.clone()));

        let result = BackendError::try_from(error_kind).expect(
            "Given an `ErrorKind` wrapping a `WriteError::Backend`, the skip-level downcast should succeed",
        );

        assert_eq!(result, backend_error);
    }

    #[test]
    fn should_fail_skip_level_downcast_to_backend_error_when_no_branch_holds_a_backend_error() {
        let error_kind = ErrorKind::Write(WriteError::failed_integrity_check(
            "SFAC-6 identity violated",
        ));
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = BackendError::try_from(error_kind).expect_err(
            "An `ErrorKind` holding no backend error should not skip-level downcast into a `BackendError`",
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_roundtrip_read_error_when_upcast_then_downcast() {
        let read_error = ReadError::MissingRecord;

        let upcast: ErrorKind = read_error.clone().into();
        let result = ReadError::try_from(upcast)
            .expect("A `ReadError` upcast into `ErrorKind` should downcast back unchanged");

        assert_eq!(result, read_error);
    }

    #[test]
    fn should_roundtrip_write_error_when_upcast_then_downcast() {
        let write_error = WriteError::failed_integrity_check("SFAC-6 identity violated");

        let upcast: ErrorKind = write_error.clone().into();
        let result = WriteError::try_from(upcast)
            .expect("A `WriteError` upcast into `ErrorKind` should downcast back unchanged");

        assert_eq!(result, write_error);
    }

    #[test]
    fn should_chain_read_display_after_caused_by_when_error_kind_wraps_read() {
        let error_kind = ErrorKind::Read(ReadError::MissingRecord);

        let expected_result = "[Read] Problem occurred during a read operation, Caused by: [MissingRecord] Requested record not found";

        let result = error_kind.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_write_display_after_caused_by_when_error_kind_wraps_write() {
        let error_kind = ErrorKind::Write(WriteError::conflicting_write("duplicate accession"));

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
    fn should_expose_read_error_as_source_when_error_kind_wraps_read() {
        let read_error = ReadError::MissingRecord;
        let error_kind = ErrorKind::Read(read_error.clone());

        let expected_result = Some(&read_error);

        let result = std::error::Error::source(&error_kind)
            .and_then(|source| source.downcast_ref::<ReadError>());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_write_error_as_source_when_error_kind_wraps_write() {
        let write_error = WriteError::failed_integrity_check("SFAC-6 identity violated");
        let error_kind = ErrorKind::Write(write_error.clone());

        let expected_result = Some(&write_error);

        let result = std::error::Error::source(&error_kind)
            .and_then(|source| source.downcast_ref::<WriteError>());

        assert_eq!(result, expected_result);
    }
}
