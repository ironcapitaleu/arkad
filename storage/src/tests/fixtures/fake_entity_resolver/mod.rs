//! # Fake Entity Resolver
//!
//! Provides [`FakeEntityResolver`], an in-memory [`EntityResolver`] test double that holds dated
//! identifier rows, and [`IdentifierRow`], one such row.

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::error::ReadError;
use crate::shared::{CompanyId, CompanyIdentifier};
use crate::traits::resolver::EntityResolver;

/// One dated identifier row: the identifier names the company from `valid_from` to `valid_to`.
///
/// Both ends are inclusive, as in the CIK range table of the SEC adapter. An end of `None` is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierRow {
    identifier: CompanyIdentifier,
    company_id: CompanyId,
    valid_from: Option<NaiveDate>,
    valid_to: Option<NaiveDate>,
}

impl IdentifierRow {
    /// Creates an [`IdentifierRow`].
    #[must_use]
    pub const fn new(
        identifier: CompanyIdentifier,
        company_id: CompanyId,
        valid_from: Option<NaiveDate>,
        valid_to: Option<NaiveDate>,
    ) -> Self {
        Self {
            identifier,
            company_id,
            valid_from,
            valid_to,
        }
    }

    /// Returns whether the row names the identifier on the date.
    fn covers(&self, identifier: &CompanyIdentifier, at: NaiveDate) -> bool {
        self.identifier == *identifier
            && self.valid_from.is_none_or(|from| from <= at)
            && self.valid_to.is_none_or(|to| at <= to)
    }
}

/// An in-memory [`EntityResolver`] test double that holds dated identifier rows.
///
/// It returns the company of the first row that covers the date. The rows of one identifier must
/// not overlap, as the trait requires.
#[derive(Debug, Default)]
pub struct FakeEntityResolver {
    rows: Vec<IdentifierRow>,
    failure: Option<ReadError>,
}

impl FakeEntityResolver {
    /// Creates a [`FakeEntityResolver`] that holds the rows.
    #[must_use]
    pub const fn new(rows: Vec<IdentifierRow>) -> Self {
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
        at: NaiveDate,
    ) -> Result<Option<CompanyId>, ReadError> {
        if let Some(failure) = self.failure.clone() {
            return Err(failure);
        }

        Ok(self
            .rows
            .iter()
            .find(|row| row.covers(identifier, at))
            .map(|row| row.company_id))
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::shared::CompanyScheme;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin + Debug + Default>() {}
    #[test]
    const fn should_implement_auto_traits_for_fake_entity_resolver() {
        implements_auto_traits::<FakeEntityResolver>();
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the creation of a date should always succeed")
    }

    fn row_of_2020() -> IdentifierRow {
        IdentifierRow::new(
            CompanyIdentifier::new(CompanyScheme::Cik, "0000315293"),
            CompanyId::from_uuid(Uuid::from_u128(1)),
            Some(date(2020, 1, 1)),
            Some(date(2020, 12, 31)),
        )
    }

    #[test]
    fn should_cover_the_date_when_the_date_is_the_first_day() {
        let row = row_of_2020();
        let identifier = CompanyIdentifier::new(CompanyScheme::Cik, "0000315293");
        let at = date(2020, 1, 1);

        let expected_result = true;

        let result = row.covers(&identifier, at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cover_the_date_when_the_date_is_the_last_day() {
        let row = row_of_2020();
        let identifier = CompanyIdentifier::new(CompanyScheme::Cik, "0000315293");
        let at = date(2020, 12, 31);

        let expected_result = true;

        let result = row.covers(&identifier, at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_cover_the_date_when_the_date_is_before_the_first_day() {
        let row = row_of_2020();
        let identifier = CompanyIdentifier::new(CompanyScheme::Cik, "0000315293");
        let at = date(2019, 12, 31);

        let expected_result = false;

        let result = row.covers(&identifier, at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cover_every_date_when_both_ends_are_open() {
        let row = IdentifierRow::new(
            CompanyIdentifier::new(CompanyScheme::Cik, "0000315293"),
            CompanyId::from_uuid(Uuid::from_u128(1)),
            None,
            None,
        );
        let identifier = CompanyIdentifier::new(CompanyScheme::Cik, "0000315293");
        let at = date(1990, 1, 1);

        let expected_result = true;

        let result = row.covers(&identifier, at);

        assert_eq!(result, expected_result);
    }
}
