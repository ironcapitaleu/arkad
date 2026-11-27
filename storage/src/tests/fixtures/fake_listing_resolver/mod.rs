//! # Fake Listing Resolver
//!
//! Provides [`FakeListingResolver`], an in-memory [`ListingResolver`] test double that holds dated
//! listing identifier rows.

use async_trait::async_trait;
use jiff::civil::Date;

use crate::error::ReadError;
use crate::shared::{ListingId, ListingIdentifier};
use crate::tests::fixtures::identifier_row::IdentifierRow;
use crate::traits::resolver::ListingResolver;

/// A dated row that names a listing.
pub type ListingIdentifierRow = IdentifierRow<ListingIdentifier, ListingId>;

/// An in-memory [`ListingResolver`] test double that holds dated identifier rows.
///
/// It returns the listing of the first row that covers the date. A row covers its first day and
/// ends before `valid_to`, as the trait requires. The rows of one identifier must not overlap.
#[derive(Debug, Default)]
pub struct FakeListingResolver {
    rows: Vec<ListingIdentifierRow>,
    failure: Option<ReadError>,
}

impl FakeListingResolver {
    /// Creates a [`FakeListingResolver`] that holds the rows.
    #[must_use]
    pub const fn new(rows: Vec<ListingIdentifierRow>) -> Self {
        Self {
            rows,
            failure: None,
        }
    }

    /// Creates a [`FakeListingResolver`] that returns the [`ReadError`] for every resolve.
    #[must_use]
    pub const fn failing(error: ReadError) -> Self {
        Self {
            rows: Vec::new(),
            failure: Some(error),
        }
    }
}

#[async_trait]
impl ListingResolver for FakeListingResolver {
    async fn resolve(
        &self,
        identifier: &ListingIdentifier,
        at: Date,
    ) -> Result<Option<ListingId>, ReadError> {
        if let Some(failure) = self.failure.clone() {
            return Err(failure);
        }

        Ok(self
            .rows
            .iter()
            .find(|row| row.covers_before_valid_to(identifier, at))
            .map(IdentifierRow::key))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_fake_listing_resolver() {
        implements_auto_traits::<FakeListingResolver>();
    }

    const fn implements_debug_and_default<T: Debug + Default>() {}
    #[test]
    const fn should_implement_debug_and_default_for_fake_listing_resolver() {
        implements_debug_and_default::<FakeListingResolver>();
    }
}
