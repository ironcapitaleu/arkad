//! # Quarter
//!
//! Provides the [`Quarter`] enum representing calendar quarters, used by
//! [`Frame`](crate::shared::financial::frame::Frame) to place a data point in time.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

/// A calendar quarter (Q1–Q4), independent of any company's fiscal year.
///
/// Distinct from fiscal quarters: a company's fiscal Q4 may fall in calendar Q3 (e.g. Apple's
/// fiscal year ends in September).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Quarter {
    /// First quarter (January - March).
    Q1,
    /// Second quarter (April - June).
    Q2,
    /// Third quarter (July - September).
    Q3,
    /// Fourth quarter (October - December).
    Q4,
}

impl Display for Quarter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Q1 => write!(f, "Q1"),
            Self::Q2 => write!(f, "Q2"),
            Self::Q3 => write!(f, "Q3"),
            Self::Q4 => write!(f, "Q4"),
        }
    }
}

impl Quarter {
    /// Parses a [`Quarter`] from a numeric string (`"1"`–`"4"`), or `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::financial::quarter::Quarter;
    ///
    /// assert_eq!(Quarter::from_number_str("3"), Some(Quarter::Q3));
    /// assert_eq!(Quarter::from_number_str("5"), None);
    /// ```
    #[must_use]
    pub fn from_number_str(s: &str) -> Option<Self> {
        match s {
            "1" => Some(Self::Q1),
            "2" => Some(Self::Q2),
            "3" => Some(Self::Q3),
            "4" => Some(Self::Q4),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_display_q1_when_quarter_is_q1() {
        let expected_result = "Q1";

        let result = Quarter::Q1.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_q3_from_number_string_when_input_is_3() {
        let expected_result = Some(Quarter::Q3);

        let result = Quarter::from_number_str("3");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_number_string_is_invalid() {
        let expected_result = None;

        let result = Quarter::from_number_str("5");

        assert_eq!(result, expected_result);
    }
}
