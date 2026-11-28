//! # Duplicate Digest Group
//!
//! Provides [`DuplicateDigestGroup`], the error raised when two group hashes of a unit have one ID.

use std::fmt::Write;

use thiserror::Error;

/// Error indicating that two group hashes of one write unit have the same group ID.
///
/// The rows of the group came in two calls of
/// [`group_hashes`](crate::shared::row_digest::group_hashes), so the digest would depend on the
/// split of the unit.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("[DuplicateDigestGroup] Two group hashes have the group ID '{group_id}'")]
pub struct DuplicateDigestGroup {
    /// The byte form of the group ID, as lowercase hex characters.
    pub group_id: String,
}

impl DuplicateDigestGroup {
    /// Creates a new [`DuplicateDigestGroup`] for the byte form of the group ID.
    #[must_use]
    pub fn new(group_id: &[u8]) -> Self {
        let group_id = group_id.iter().fold(
            String::with_capacity(group_id.len() * 2),
            |mut hex, byte| {
                // Deviation: the result is ignored. Writing to a `String` never fails.
                let _ = write!(hex, "{byte:02x}");
                hex
            },
        );
        Self { group_id }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_duplicate_digest_group() {
        implements_auto_traits::<DuplicateDigestGroup>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + std::error::Error,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_duplicate_digest_group() {
        implements_value_traits::<DuplicateDigestGroup>();
    }

    #[test]
    fn should_format_display_as_expected_for_a_duplicate_digest_group() {
        let group_id = [0x01, 0, 0, 0, 1, b'a'];
        let error = DuplicateDigestGroup::new(&group_id);

        let expected_result =
            "[DuplicateDigestGroup] Two group hashes have the group ID '010000000161'";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
