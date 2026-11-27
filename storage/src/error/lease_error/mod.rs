//! # Lease Error
//!
//! Provides [`LeaseError`], the error raised when a [`LeaseRepository`](crate::LeaseRepository)
//! cannot take or read a lease.

use jiff::tz::Offset;
use thiserror::Error;

use super::ErrorKind;
use super::backend_error::BackendError;
use crate::shared::stored_timestamp::fraction_digits;
use crate::shared::{JobKind, Lease};

#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Error occurring while taking or reading a lease.
pub enum LeaseError {
    /// A plan holds the lease of the job kind, and the lease has not expired.
    #[error(
        "[LeaseHeld] Lease of job kind '{kind}' is held by plan '{}' until '{:.*}'",
        .lease.plan_id,
        fraction_digits(.lease.expires_at),
        .lease.expires_at.display_with_offset(Offset::UTC)
    )]
    LeaseHeld {
        /// The job kind of the lease.
        kind: JobKind,
        /// The lease that holds it.
        lease: Lease,
    },

    /// The lease operation failed at the storage backend.
    ///
    /// Passes through: prints the inner error as it is, because this layer adds no fact.
    #[error(transparent)]
    Backend(BackendError),
}

impl From<BackendError> for LeaseError {
    /// Converts a [`BackendError`] into a [`LeaseError::Backend`] variant.
    fn from(error: BackendError) -> Self {
        Self::Backend(error)
    }
}

impl TryFrom<LeaseError> for BackendError {
    type Error = ErrorKind;

    /// Extracts the [`BackendError`] from a [`LeaseError::Backend`].
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::DowncastNotPossible`] if the value is not a [`LeaseError::Backend`].
    fn try_from(value: LeaseError) -> Result<Self, Self::Error> {
        match value {
            LeaseError::Backend(backend) => Ok(backend),
            LeaseError::LeaseHeld { .. } => Err(ErrorKind::DowncastNotPossible),
        }
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_lease_error() {
        implements_auto_traits::<LeaseError>();
    }

    #[test]
    fn should_name_the_job_kind_the_plan_and_the_expiry_when_the_lease_is_held() {
        let error = LeaseError::LeaseHeld {
            kind: JobKind::new("transform-notes")
                .expect("Given a valid hardcoded job kind, the creation should always succeed"),
            lease: Lease {
                plan_id: "2026-11-21T180000.000Z".to_owned(),
                expires_at: Timestamp::from_second(1_795_341_600)
                    .expect("Given a hardcoded timestamp in range, the time should always build"),
            },
        };

        let expected_result = "[LeaseHeld] Lease of job kind 'transform-notes' is held by plan '2026-11-21T180000.000Z' until '2026-11-22T10:00:00+00:00'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_expiry_with_milliseconds_when_the_expiry_has_a_fraction() {
        let error = LeaseError::LeaseHeld {
            kind: JobKind::new("transform-notes")
                .expect("Given a valid hardcoded job kind, the creation should always succeed"),
            lease: Lease {
                plan_id: "plan-a".to_owned(),
                expires_at: Timestamp::new(1_795_341_600, 500_000_000)
                    .expect("Given a hardcoded timestamp in range, the time should always build"),
            },
        };

        let expected_result = "[LeaseHeld] Lease of job kind 'transform-notes' is held by plan 'plan-a' until '2026-11-22T10:00:00.500+00:00'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_the_backend_error_as_it_is_when_the_backend_fails() {
        let error = LeaseError::Backend(BackendError::unreachable_storage("timeout"));

        let expected_result =
            "[UnreachableStorage] Storage backend is not reachable, Reason: 'timeout'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_downcast_to_backend_error_when_the_lease_is_held() {
        let error = LeaseError::LeaseHeld {
            kind: JobKind::new("series")
                .expect("Given a valid hardcoded job kind, the creation should always succeed"),
            lease: Lease {
                plan_id: "2026-11-21T180000.000Z".to_owned(),
                expires_at: Timestamp::UNIX_EPOCH,
            },
        };

        let expected_result = ErrorKind::DowncastNotPossible;

        let result = BackendError::try_from(error)
            .expect_err("A held lease holds no backend error, so the downcast should fail");

        assert_eq!(result, expected_result);
    }
}
