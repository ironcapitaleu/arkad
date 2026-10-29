//! # Year
//!
//! Provides the [`Year`] type for a calendar year.

use std::fmt::{self, Display, Formatter};

use super::is_ascii_digits;

/// A calendar year, not a company's fiscal year.
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
        is_ascii_digits(number, 4)
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
    fn should_return_none_when_number_has_letters() {
        let expected_result = (None, None);

        let result = (
            Year::from_number_str("Dumb"),
            Year::from_number_str("Dumbo"),
        );

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_four_digits_when_year_is_2009() {
        let expected_result = "2009";

        let result = Year(2009).to_string();

        assert_eq!(result, expected_result);
    }
}
