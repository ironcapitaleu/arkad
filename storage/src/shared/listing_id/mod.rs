//! # Listing ID
//!
//! Provides [`ListingId`], the internal key of a listing. Every identifier row and price of a
//! listing keys on it.

use std::fmt::{self, Display, Formatter};

use uuid::Uuid;

/// The internal key of a listing: an opaque UUID.
///
/// A listing is one security that trades on one exchange. The load assigns a [`ListingId`] once
/// per listing, and nothing reuses or changes it. The value carries no meaning: a ticker or a FIGI
/// is an identifier of the listing, not its key. To find the key of an identifier, ask a
/// [`ListingResolver`](crate::ListingResolver).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListingId(Uuid);

impl ListingId {
    /// Wraps a UUID that the load assigned to a listing, or that a store read back.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use storage::ListingId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::from_u128(0x0193_0000_0000_7000_8000_0000_0000_0001);
    ///
    /// let expected_result = uuid;
    ///
    /// let result = ListingId::from_uuid(uuid).as_uuid();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the UUID, for a store that writes the key.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Display for ListingId {
    /// Writes the UUID in its hyphenated lowercase form.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
    const fn should_implement_auto_traits_for_listing_id() {
        implements_auto_traits::<ListingId>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_listing_id() {
        implements_value_traits::<ListingId>();
    }

    #[test]
    fn should_return_the_wrapped_uuid_for_a_listing_id() {
        let uuid = Uuid::from_u128(1);

        let expected_result = Uuid::from_u128(1);

        let result = ListingId::from_uuid(uuid).as_uuid();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_hyphenated_uuid_for_a_listing_id() {
        let listing_id = ListingId::from_uuid(Uuid::from_u128(1));

        let expected_result = "00000000-0000-0000-0000-000000000001";

        let result = listing_id.to_string();

        assert_eq!(result, expected_result);
    }
}
