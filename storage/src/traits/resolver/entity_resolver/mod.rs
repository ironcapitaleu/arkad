//! # Entity Resolver
//!
//! Provides [`EntityResolver`], the port that returns the [`CompanyId`] of a company identifier at
//! a date.
//!
//! ## Usage
//!
//! A load step takes the port and never names the engine behind it:
//!
//! ```rust
//! use chrono::NaiveDate;
//! use storage::{CompanyId, CompanyIdentifier, EntityResolver, ReadError};
//!
//! async fn company_of_filing<R: EntityResolver>(
//!     resolver: &R,
//!     identifier: &CompanyIdentifier,
//!     period_end: NaiveDate,
//! ) -> Result<Option<CompanyId>, ReadError> {
//!     resolver.resolve(identifier, period_end).await
//! }
//! ```

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::error::ReadError;
use crate::shared::{CompanyId, CompanyIdentifier};

/// Returns the [`CompanyId`] of a company identifier at a date.
///
/// An implementor holds dated identifier rows: a `(scheme, value)` pair, a [`CompanyId`], and a
/// validity from one date to another. It returns at most one [`CompanyId`] for one identifier and
/// one date. If its rows give more than one, as two sources that disagree can, a claim
/// resolution spec picks the row. ADR 0001 lists that spec as an open question.
///
/// The implementor fixes what the date means. The caller passes a date on that axis:
///
/// - The CIK range table of the SEC adapter is on the period end of a filing. A row covers both
///   its first and its last day (Linear document "Entity Resolution: the Company Key, LEI, and
///   Dated CIKs: Findings", Decision 2).
/// - The identifier rows of the metadata store are on the start of the record that gives each
///   value. A row covers its first day, and ends before `valid_to`
///   (`sec/design/data_model/identifier_model.md`, "Point-in-Time Reads").
///
/// The port covers the company level only. Securities and listings have resolvers of their own
/// (ADR 0009).
#[async_trait]
pub trait EntityResolver: Send + Sync {
    /// Returns the [`CompanyId`] of the row of the identifier that covers the date.
    ///
    /// Returns `Ok(None)` if the identifier has no row, or if no row of it covers the date.
    /// `Ok(None)` is an answer, not a failure.
    ///
    /// # Errors
    ///
    /// Returns a [`ReadError`] if the read of the rows fails.
    async fn resolve(
        &self,
        identifier: &CompanyIdentifier,
        at: NaiveDate,
    ) -> Result<Option<CompanyId>, ReadError>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::error::BackendError;
    use crate::shared::CompanyScheme;
    use crate::tests::fixtures::fake_entity_resolver::{FakeEntityResolver, IdentifierRow};

    /// The dates of the two CIKs of Alphabet come from `cik_map.csv`, attached to STA-285 under
    /// "Final: all scripts, results, CIK map, and RUN.md (2026-10-17)". The Linear document
    /// "Entity Resolution: the Company Key, LEI, and Dated CIKs: Findings" names the two CIKs.
    const GOOGLE_CIK: &str = "0001288776";
    const ALPHABET_CIK: &str = "0001652044";

    fn alphabet() -> CompanyId {
        CompanyId::from_uuid(Uuid::from_u128(1))
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the creation of a date should always succeed")
    }

    fn cik(value: &str) -> CompanyIdentifier {
        CompanyIdentifier::new(CompanyScheme::Cik, value)
    }

    /// The rows of the CIK map: Google up to the period end 2015-06-30, and Alphabet from the
    /// period end 2015-09-30, with an open end.
    fn alphabet_resolver() -> FakeEntityResolver {
        FakeEntityResolver::new(vec![
            IdentifierRow::new(cik(GOOGLE_CIK), alphabet(), None, Some(date(2015, 6, 30))),
            IdentifierRow::new(cik(ALPHABET_CIK), alphabet(), Some(date(2015, 9, 30)), None),
        ])
    }

    const fn implements_entity_resolver<T: EntityResolver>() {}
    #[test]
    const fn should_implement_entity_resolver_for_fake_entity_resolver() {
        implements_entity_resolver::<FakeEntityResolver>();
    }

    #[tokio::test]
    async fn should_return_the_company_id_when_one_row_covers_the_date() {
        let resolver = alphabet_resolver();
        let identifier = cik(ALPHABET_CIK);
        let at = date(2024, 12, 31);

        let expected_result = Some(alphabet());

        let result = resolver
            .resolve(&identifier, at)
            .await
            .expect("Given a fake that never fails, the resolve should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_identifier_has_no_row() {
        let resolver = alphabet_resolver();
        let identifier = cik("0000320193");
        let at = date(2024, 12, 31);

        let expected_result = None;

        let result = resolver
            .resolve(&identifier, at)
            .await
            .expect("Given a fake that never fails, the resolve should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_date_is_outside_every_range() {
        let resolver = alphabet_resolver();
        let identifier = cik(GOOGLE_CIK);
        let at = date(2015, 9, 30);

        let expected_result = None;

        let result = resolver
            .resolve(&identifier, at)
            .await
            .expect("Given a fake that never fails, the resolve should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_none_when_the_value_matches_under_another_scheme() {
        let resolver = alphabet_resolver();
        let identifier = CompanyIdentifier::new(CompanyScheme::Lei, ALPHABET_CIK);
        let at = date(2024, 12, 31);

        let expected_result = None;

        let result = resolver
            .resolve(&identifier, at)
            .await
            .expect("Given a fake that never fails, the resolve should always succeed");

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_resolve_both_ciks_to_one_company_id_for_alphabet() {
        let resolver = alphabet_resolver();
        let google = cik(GOOGLE_CIK);
        let google_last_period_end = date(2015, 6, 30);
        let alphabet_first_period_end = date(2015, 9, 30);

        let expected_result = (Some(alphabet()), Some(alphabet()));

        let result = (
            resolver
                .resolve(&google, google_last_period_end)
                .await
                .expect("Given a fake that never fails, the resolve should always succeed"),
            resolver
                .resolve(&cik(ALPHABET_CIK), alphabet_first_period_end)
                .await
                .expect("Given a fake that never fails, the resolve should always succeed"),
        );

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_the_error_when_the_read_of_the_rows_fails() {
        let error = ReadError::Backend(BackendError::unreachable_storage("timeout"));
        let resolver: Box<dyn EntityResolver> =
            Box::new(FakeEntityResolver::failing(error.clone()));
        let identifier = cik(ALPHABET_CIK);
        let at = date(2024, 12, 31);

        let expected_result = error;

        let result = resolver.resolve(&identifier, at).await.expect_err(
            "Given a fake initialized to fail, the resolve should always return that error",
        );

        assert_eq!(result, expected_result);
    }
}
