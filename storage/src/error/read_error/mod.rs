//! # Read Error
//!
//! Provides [`ReadError`], the error raised when the store cannot complete a read.
//!
//! ## Usage
//!
//! ```rust
//! use storage::ReadError;
//!
//! let _err = ReadError::MissingRecord;
//! ```

use thiserror::Error;

use super::ErrorKind;
use super::backend_error::BackendError;

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Error occurring while reading from the store.
///
/// Separates the kinds of read failure so a caller can tell them apart. A backend failure keeps its
/// own type, so the cause survives the wrapping.
pub enum ReadError {
    /// The requested record is not present in the store.
    #[error("[MissingRecord] Requested record not found")]
    MissingRecord,

    /// The read failed at the storage backend.
    #[error("[Backend] Storage backend error occurred, Caused by: {0}")]
    Backend(#[source] BackendError),
}

impl From<BackendError> for ReadError {
    /// Converts a [`BackendError`] into a [`ReadError::Backend`] variant.
    fn from(error: BackendError) -> Self {
        Self::Backend(error)
    }
}

impl TryFrom<ReadError> for BackendError {
    type Error = ErrorKind;

    /// Extracts the [`BackendError`] from a [`ReadError::Backend`].
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not a [`ReadError::Backend`].
    fn try_from(value: ReadError) -> Result<Self, Self::Error> {
        match value {
            ReadError::Backend(backend) => Ok(backend),
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
    const fn should_be_able_to_rely_auto_trait_implementation_when_using_read_error() {
        implements_auto_traits::<ReadError>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_using_read_error() {
        implements_send::<ReadError>();
    }

    #[test]
    const fn should_implement_sync_when_using_read_error() {
        implements_sync::<ReadError>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_able_to_rely_on_error_being_sized_when_using_read_error() {
        implements_sized::<ReadError>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_be_able_to_rely_on_hash_implementation_when_using_read_error() {
        implements_hash::<ReadError>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_eq_implementation_when_using_read_error() {
        implements_partial_eq::<ReadError>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_be_able_to_rely_on_eq_implementation_when_using_read_error() {
        implements_eq::<ReadError>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_be_able_to_rely_on_partial_ord_implementation_when_using_read_error() {
        implements_partial_ord::<ReadError>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_be_able_to_rely_on_ord_implementation_when_using_read_error() {
        implements_ord::<ReadError>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_be_able_to_rely_on_debug_implementation_when_using_read_error() {
        implements_debug::<ReadError>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_be_able_to_rely_on_clone_implementation_when_using_read_error() {
        implements_clone::<ReadError>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_be_able_to_rely_on_unpin_implementation_when_using_read_error() {
        implements_unpin::<ReadError>();
    }

    #[test]
    fn should_wrap_backend_error_into_backend_variant_when_converting_from_backend_error() {
        let backend_error = BackendError::unreachable_storage("connection reset");
        let expected_result = ReadError::Backend(backend_error.clone());

        let result = ReadError::from(backend_error);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_downcast_to_backend_error_when_read_error_is_a_backend_variant() {
        let backend_error = BackendError::unreachable_storage("timeout");
        let read_error = ReadError::Backend(backend_error.clone());

        let result = BackendError::try_from(read_error)
            .expect("Given a `ReadError::Backend`, the downcast to `BackendError` should succeed");

        assert_eq!(result, backend_error);
    }

    #[test]
    fn should_fail_downcast_to_backend_error_when_read_error_is_not_a_backend_variant() {
        let read_error = ReadError::MissingRecord;
        let expected_result = ErrorKind::DowncastNotPossible;

        let result = BackendError::try_from(read_error)
            .expect_err("A non-backend `ReadError` should not downcast into a `BackendError`");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_format_display_with_bracketed_name_when_record_is_missing() {
        let error = ReadError::MissingRecord;

        let expected_result = "[MissingRecord] Requested record not found";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_chain_backend_display_after_caused_by_when_read_error_wraps_backend() {
        let error = ReadError::Backend(BackendError::failed_operation("statement timeout"));

        let expected_result = "[Backend] Storage backend error occurred, Caused by: [FailedOperation] Storage backend operation failed, Reason: 'statement timeout'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_expose_backend_error_as_source_when_read_error_wraps_backend() {
        let backend_error = BackendError::unreachable_storage("timeout");
        let error = ReadError::Backend(backend_error.clone());

        let expected_result = Some(&backend_error);

        let result = std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<BackendError>());

        assert_eq!(result, expected_result);
    }
}
