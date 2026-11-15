//! # Listing Resolver
//!
//! Provides [`ListingResolver`], the port that returns the [`ListingId`] of a listing identifier at
//! a date.
//!
//! ## Usage
//!
//! A load step takes the port and never names the engine behind it:
//!
//! ```rust
//! use chrono::NaiveDate;
//! use storage::{ListingId, ListingIdentifier, ListingResolver, ReadError};
//!
//! async fn listing_of_price<R: ListingResolver>(
//!     resolver: &R,
//!     identifier: &ListingIdentifier,
//!     trading_day: NaiveDate,
//! ) -> Result<Option<ListingId>, ReadError> {
//!     resolver.resolve(identifier, trading_day).await
//! }
//! ```

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::error::ReadError;
use crate::shared::{ListingId, ListingIdentifier};

/// Returns the [`ListingId`] of a listing identifier at a date.
///
/// An implementor holds dated identifier rows: a `(scheme, value)` pair, a [`ListingId`], and a
/// validity from one date to another. A row covers its first day and ends before `valid_to`, the
/// day the next record or an end rule starts. It returns at most one [`ListingId`] for
/// one identifier and one date. If two sources that disagree give two rows for one date, the
/// implementor picks one. The port does not fix which.
///
/// Two identifiers resolve to one [`ListingId`] only if rows of one listing hold both. A ticker
/// change with no row that links the two tickers gives two listings.
///
/// The port covers the listing level only. It takes no company or security identifier.
#[async_trait]
pub trait ListingResolver: Send + Sync {
    /// Returns the [`ListingId`] of the row of the identifier that covers the date.
    ///
    /// Returns `Ok(None)` if the identifier has no row, or if no row of it covers the date.
    /// `Ok(None)` is an answer, not a failure.
    ///
    /// # Errors
    ///
    /// Returns a [`ReadError`] if the read of the rows fails.
    async fn resolve(
        &self,
        identifier: &ListingIdentifier,
        at: NaiveDate,
    ) -> Result<Option<ListingId>, ReadError>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::error::BackendError;
    use crate::shared::ListingScheme;
    use crate::tests::fixtures::fake_listing_resolver::{
        FakeListingResolver, ListingIdentifierRow,
    };

    /// The 8-K that Meta filed on 2022-06-01 tags `dei:TradingSymbol` as `FB`. The next filing, an
    /// 8-K accepted on 2022-07-27, tags `META`. So with SEC data only, the `FB` row ends and the
    /// `META` row starts on 2022-07-27. The market changed the ticker on 2022-06-09, between the
    /// two filings.
    const FB: &str = "FB";
    const META: &str = "META";

    /// A placeholder FIGI that names the listing under both tickers.
    const FIGI: &str = "BBG000000001";

    fn fb_listing() -> ListingId {
        ListingId::from_uuid(Uuid::from_u128(1))
    }

    fn meta_listing() -> ListingId {
        ListingId::from_uuid(Uuid::from_u128(2))
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the creation of a date should always succeed")
    }

    fn ticker(value: &str) -> ListingIdentifier {
        ListingIdentifier::new(ListingScheme::UsTicker, value)
    }

    fn figi(value: &str) -> ListingIdentifier {
        ListingIdentifier::new(ListingScheme::Figi, value)
    }

    /// The rows that SEC data alone gives: no key links `FB` and `META`, so each ticker has a
    /// listing of its own.
    fn resolver_without_figi() -> FakeListingResolver {
        FakeListingResolver::new(vec![
            ListingIdentifierRow::new(ticker(FB), fb_listing(), None, Some(date(2022, 7, 27))),
            ListingIdentifierRow::new(ticker(META), meta_listing(), Some(date(2022, 7, 27)), None),
        ])
    }

    /// The rows after the load links the two tickers through a `figi` row: all three name the
    /// older listing.
    fn resolver_with_figi() -> FakeListingResolver {
        FakeListingResolver::new(vec![
            ListingIdentifierRow::new(ticker(FB), fb_listing(), None, Some(date(2022, 7, 27))),
            ListingIdentifierRow::new(ticker(META), fb_listing(), Some(date(2022, 7, 27)), None),
            ListingIdentifierRow::new(figi(FIGI), fb_listing(), None, None),
        ])
    }

    const fn implements_listing_resolver<T: ListingResolver>() {}
    #[test]
    const fn should_implement_listing_resolver_for_fake_listing_resolver() {
        implements_listing_resolver::<FakeListingResolver>();
    }

    #[tokio::test]
    async fn should_resolve_fb_and_meta_to_two_listings_when_no_figi_row_links_them() {
        let resolver = resolver_without_figi();
        let before_the_change = date(2022, 6, 1);
        let after_the_change = date(2022, 7, 27);

        let expected_result = (Some(fb_listing()), Some(meta_listing()));

        let result = (
            resolver
                .resolve(&ticker(FB), before_the_change)
                .await
                .expect("Given a fake that never fails, the resolve should always succeed"),
            resolver
                .resolve(&ticker(META), after_the_change)
                .await
                .expect("Given a fake that never fails, the resolve should always succeed"),
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_resolve_fb_meta_and_the_figi_to_one_listing_when_their_rows_share_a_listing_id()
    {
        let resolver = resolver_with_figi();
        let before_the_change = date(2022, 6, 1);
        let after_the_change = date(2022, 7, 27);

        let expected_result = (Some(fb_listing()), Some(fb_listing()), Some(fb_listing()));

        let result = (
            resolver
                .resolve(&ticker(FB), before_the_change)
                .await
                .expect("Given a fake that never fails, the resolve should always succeed"),
            resolver
                .resolve(&ticker(META), after_the_change)
                .await
                .expect("Given a fake that never fails, the resolve should always succeed"),
            resolver
                .resolve(&figi(FIGI), after_the_change)
                .await
                .expect("Given a fake that never fails, the resolve should always succeed"),
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_listing_id_when_a_figi_row_covers_the_date() {
        let resolver = resolver_with_figi();
        let identifier = figi(FIGI);
        let at = date(2024, 12, 31);

        let expected_result = Some(fb_listing());

        let result = resolver
            .resolve(&identifier, at)
            .await
            .expect("Given a fake that never fails, the resolve should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_date_is_valid_to_of_the_only_row() {
        let resolver = resolver_without_figi();
        let identifier = ticker(FB);
        let at = date(2022, 7, 27);

        let expected_result = None;

        let result = resolver
            .resolve(&identifier, at)
            .await
            .expect("Given a fake that never fails, the resolve should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_identifier_has_no_row() {
        let resolver = resolver_without_figi();
        let identifier = ticker("AAPL");
        let at = date(2024, 12, 31);

        let expected_result = None;

        let result = resolver
            .resolve(&identifier, at)
            .await
            .expect("Given a fake that never fails, the resolve should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_value_matches_under_another_scheme() {
        let resolver = resolver_without_figi();
        let identifier = figi(META);
        let at = date(2024, 12, 31);

        let expected_result = None;

        let result = resolver
            .resolve(&identifier, at)
            .await
            .expect("Given a fake that never fails, the resolve should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_error_when_the_read_of_the_rows_fails() {
        let error = ReadError::Backend(BackendError::unreachable_storage("timeout"));
        let resolver: Box<dyn ListingResolver> =
            Box::new(FakeListingResolver::failing(error.clone()));
        let identifier = ticker(META);
        let at = date(2024, 12, 31);

        let expected_result = error;

        let result = resolver.resolve(&identifier, at).await.expect_err(
            "Given a fake initialized to fail, the resolve should always return that error",
        );

        assert_eq!(result, expected_result);
    }
}
