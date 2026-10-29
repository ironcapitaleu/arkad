//! # Year
//!
//! Provides the calendar [`Year`] of one Notes file.

use std::fmt::{self, Display, Formatter};

/// A calendar year, not a company's fiscal year.
///
/// A Notes file holds the filings with a filed date in its period, so its year is a calendar year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Year(pub(super) u16);

impl Year {
    /// Parses a [`Year`] from its four digits, or returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_period::year::Year;
    ///
    /// let year = Year::from_number_str("2009").expect("A hardcoded four-digit year should always parse");
    ///
    /// let expected_result = "2009";
    ///
    /// let result = year.to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn from_number_str(number: &str) -> Option<Self> {
        let is_four_digits = number.len() == 4 && number.bytes().all(|byte| byte.is_ascii_digit());
        is_four_digits
            .then(|| number.parse().ok().map(Self))
            .flatten()
    }
}

impl Display for Year {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_none_when_number_has_a_sign() {
        let expected_result = None;

        let result = Year::from_number_str("+202");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_four_digits_when_year_is_2009() {
        let expected_result = "2009";

        let result = Year(2009).to_string();

        assert_eq!(result, expected_result);
    }
}
