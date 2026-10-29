//! # Month
//!
//! Provides the [`Month`] type for a month of a calendar year.

use std::fmt::{self, Display, Formatter};

use super::is_ascii_digits;

/// A month of a calendar year, written as two digits (`01` to `12`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Month(pub(super) chrono::Month);

impl Month {
    /// Parses a [`Month`] from its two-digit number, `"01"` to `"12"`, or returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_period::month::Month;
    ///
    /// let month = Month::from_number_str("09").expect("A hardcoded month number should always parse");
    ///
    /// let expected_result = "09";
    ///
    /// let result = month.to_string();
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn from_number_str(number: &str) -> Option<Self> {
        let number: u8 = is_ascii_digits(number, 2)
            .then(|| number.parse().ok())
            .flatten()?;
        chrono::Month::try_from(number).ok().map(Self)
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:02}", self.0.number_from_month())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_return_none_when_number_is_outside_01_to_12() {
        let expected_result = (None, None);

        let result = (Month::from_number_str("00"), Month::from_number_str("13"));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_number_has_one_digit() {
        let expected_result = None;

        let result = Month::from_number_str("1");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_number_has_a_sign() {
        let expected_result = None;

        let result = Month::from_number_str("+1");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_january_before_december_for_months() {
        let expected_result = true;

        let result = Month(chrono::Month::January) < Month(chrono::Month::December);

        assert_eq!(result, expected_result);
    }
}
