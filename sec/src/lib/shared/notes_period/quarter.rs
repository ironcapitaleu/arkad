//! # Quarter
//!
//! Provides the [`Quarter`] of a calendar year that one quarterly Notes file covers.

use std::fmt::{self, Display, Formatter};

/// A quarter of a calendar year, written as the SEC writes it in Notes file names (`q1` to `q4`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Quarter {
    /// January to March.
    Q1,
    /// April to June.
    Q2,
    /// July to September.
    Q3,
    /// October to December.
    Q4,
}

impl Quarter {
    /// Parses a [`Quarter`] from its number, `"1"` to `"4"`, or returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::notes_period::quarter::Quarter;
    ///
    /// let expected_result = Some(Quarter::Q3);
    ///
    /// let result = Quarter::from_number_str("3");
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    #[must_use]
    pub fn from_number_str(number: &str) -> Option<Self> {
        match number {
            "1" => Some(Self::Q1),
            "2" => Some(Self::Q2),
            "3" => Some(Self::Q3),
            "4" => Some(Self::Q4),
            _ => None,
        }
    }
}

impl Display for Quarter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Q1 => write!(f, "q1"),
            Self::Q2 => write!(f, "q2"),
            Self::Q3 => write!(f, "q3"),
            Self::Q4 => write!(f, "q4"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_display_the_lowercase_sec_form_when_quarter_is_q4() {
        let expected_result = "q4";

        let result = Quarter::Q4.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_number_is_outside_1_to_4() {
        let expected_result = None;

        let result = Quarter::from_number_str("5");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_q1_before_q4_for_quarters() {
        let expected_result = true;

        let result = Quarter::Q1 < Quarter::Q4;

        assert_eq!(result, expected_result);
    }
}
