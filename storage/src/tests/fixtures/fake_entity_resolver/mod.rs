//! # Fake Entity Resolver
//!
//! Provides [`FakeEntityResolver`], an in-memory [`EntityResolver`] test double that holds dated
//! company identifier rows.

use async_trait::async_trait;
use jiff::civil::Date;

use crate::error::ReadError;
use crate::shared::{CompanyId, CompanyIdentifier};
use crate::tests::fixtures::identifier_row::IdentifierRow;
use crate::traits::resolver::EntityResolver;

/// A dated row that names a company.
pub type CompanyIdentifierRow = IdentifierRow<CompanyIdentifier, CompanyId>;

/// An in-memory [`EntityResolver`] test double that holds dated identifier rows.
///
/// It returns the company of the first row that covers the date. A row covers both its first day
/// and `valid_to`, as in the CIK range table of the SEC adapter. The rows of one identifier must
/// not overlap.
#[derive(Debug, Default)]
pub struct FakeEntityResolver {
    rows: Vec<CompanyIdentifierRow>,
    failure: Option<ReadError>,
}

impl FakeEntityResolver {
    /// Creates a [`FakeEntityResolver`] that holds the rows.
    #[must_use]
    pub const fn new(rows: Vec<CompanyIdentifierRow>) -> Self {
        Self {
            rows,
            failure: None,
        }
    }

    /// Creates a [`FakeEntityResolver`] that returns the [`ReadError`] for every resolve.
    #[must_use]
    pub const fn failing(error: ReadError) -> Self {
        Self {
            rows: Vec::new(),
            failure: Some(error),
        }
    }
}

#[async_trait]
impl EntityResolver for FakeEntityResolver {
    async fn resolve(
        &self,
        identifier: &CompanyIdentifier,
        at: Date,
    ) -> Result<Option<CompanyId>, ReadError> {
        if let Some(failure) = self.failure.clone() {
            return Err(failure);
        }

        Ok(self
            .rows
            .iter()
            .find(|row| row.covers_through_valid_to(identifier, at))
            .map(IdentifierRow::key))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_fake_entity_resolver() {
        implements_auto_traits::<FakeEntityResolver>();
    }

    const fn implements_debug_and_default<T: Debug + Default>() {}
    #[test]
    const fn should_implement_debug_and_default_for_fake_entity_resolver() {
        implements_debug_and_default::<FakeEntityResolver>();
    }
}
