//! # Identifier Row
//!
//! Provides [`IdentifierRow`], one dated identifier row of any level of identity. The fake
//! resolvers hold such rows.

use chrono::NaiveDate;

/// One dated identifier row: the identifier names the subject with the key `key` from
/// `valid_from` to `valid_to`.
///
/// An end of `None` is open. The two coverage methods differ in the last day only, because the
/// stores that a resolver reads differ in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierRow<I, K> {
    identifier: I,
    key: K,
    valid_from: Option<NaiveDate>,
    valid_to: Option<NaiveDate>,
}

impl<I: PartialEq, K: Copy> IdentifierRow<I, K> {
    /// Creates an [`IdentifierRow`].
    #[must_use]
    pub const fn new(
        identifier: I,
        key: K,
        valid_from: Option<NaiveDate>,
        valid_to: Option<NaiveDate>,
    ) -> Self {
        Self {
            identifier,
            key,
            valid_from,
            valid_to,
        }
    }

    /// Returns the key of the subject that the row names.
    pub const fn key(&self) -> K {
        self.key
    }

    /// Returns whether the row names the identifier on the date, with `valid_to` as its last day.
    ///
    /// The CIK range table of the SEC adapter covers both ends.
    pub fn covers_through_valid_to(&self, identifier: &I, at: NaiveDate) -> bool {
        self.starts_by(identifier, at) && self.valid_to.is_none_or(|to| at <= to)
    }

    /// Returns whether the row names the identifier on the date, with the row ending before
    /// `valid_to`.
    ///
    /// The identifier rows of the metadata store cover their first day and end before `valid_to`,
    /// because `valid_to` is the start of the next record.
    pub fn covers_before_valid_to(&self, identifier: &I, at: NaiveDate) -> bool {
        self.starts_by(identifier, at) && self.valid_to.is_none_or(|to| at < to)
    }

    fn starts_by(&self, identifier: &I, at: NaiveDate) -> bool {
        self.identifier == *identifier && self.valid_from.is_none_or(|from| from <= at)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day)
            .expect("Given a hardcoded calendar date, the creation of a date should always succeed")
    }

    fn row_of_2020() -> IdentifierRow<&'static str, u8> {
        IdentifierRow::new(
            "cik:0000315293",
            1,
            Some(date(2020, 1, 1)),
            Some(date(2020, 12, 31)),
        )
    }

    #[test]
    fn should_return_the_key_for_an_identifier_row() {
        let row = row_of_2020();

        let expected_result = 1;

        let result = row.key();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cover_the_date_through_valid_to_when_the_date_is_the_first_day() {
        let row = row_of_2020();
        let at = date(2020, 1, 1);

        let expected_result = true;

        let result = row.covers_through_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cover_the_date_through_valid_to_when_the_date_is_valid_to() {
        let row = row_of_2020();
        let at = date(2020, 12, 31);

        let expected_result = true;

        let result = row.covers_through_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_cover_the_date_through_valid_to_when_the_date_is_before_the_first_day() {
        let row = row_of_2020();
        let at = date(2019, 12, 31);

        let expected_result = false;

        let result = row.covers_through_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_cover_the_date_through_valid_to_when_the_date_is_after_valid_to() {
        let row = row_of_2020();
        let at = date(2021, 1, 1);

        let expected_result = false;

        let result = row.covers_through_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_cover_the_date_through_valid_to_when_the_identifier_differs() {
        let row = row_of_2020();
        let at = date(2020, 6, 30);

        let expected_result = false;

        let result = row.covers_through_valid_to(&"cik:0000858877", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cover_every_date_through_valid_to_when_both_ends_are_open() {
        let row = IdentifierRow::new("cik:0000315293", 1_u8, None, None);
        let at = date(1990, 1, 1);

        let expected_result = true;

        let result = row.covers_through_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cover_the_date_before_valid_to_when_the_date_is_the_first_day() {
        let row = row_of_2020();
        let at = date(2020, 1, 1);

        let expected_result = true;

        let result = row.covers_before_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cover_the_date_before_valid_to_when_the_date_is_the_day_before_valid_to() {
        let row = row_of_2020();
        let at = date(2020, 12, 30);

        let expected_result = true;

        let result = row.covers_before_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_cover_the_date_before_valid_to_when_the_date_is_valid_to() {
        let row = row_of_2020();
        let at = date(2020, 12, 31);

        let expected_result = false;

        let result = row.covers_before_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_cover_the_date_before_valid_to_when_the_date_is_before_the_first_day() {
        let row = row_of_2020();
        let at = date(2019, 12, 31);

        let expected_result = false;

        let result = row.covers_before_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_cover_the_date_before_valid_to_when_the_identifier_differs() {
        let row = row_of_2020();
        let at = date(2020, 6, 30);

        let expected_result = false;

        let result = row.covers_before_valid_to(&"cik:0000858877", at);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_cover_every_date_before_valid_to_when_both_ends_are_open() {
        let row = IdentifierRow::new("cik:0000315293", 1_u8, None, None);
        let at = date(2100, 1, 1);

        let expected_result = true;

        let result = row.covers_before_valid_to(&"cik:0000315293", at);

        assert_eq!(result, expected_result);
    }
}
