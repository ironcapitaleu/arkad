//! # Quarter Module
//!
//! Provides the [`Quarter`] enum representing calendar quarters.
//! Used in [`Frame`](crate::shared::financial::frame::Frame) to identify
//! which calendar quarter a data point belongs to.

use std::fmt;

/// A calendar quarter (Q1 through Q4).
///
/// Represents a standard calendar quarter, independent of a company's fiscal year.
/// A company's fiscal Q4 may correspond to calendar Q3 (e.g., Apple's fiscal year ends in September).
///
/// # Example
/// ```
/// use sec::shared::financial::quarter::Quarter;
///
/// let q = Quarter::Q1;
/// assert_eq!(q.to_string(), "Q1");
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
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

impl fmt::Display for Quarter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Q1 => write!(f, "Q1"),
            Self::Q2 => write!(f, "Q2"),
            Self::Q3 => write!(f, "Q3"),
            Self::Q4 => write!(f, "Q4"),
        }
    }
}

impl Quarter {
    /// Attempts to parse a [`Quarter`] from a numeric string (e.g., `"1"` → `Q1`).
    ///
    /// # Errors
    ///
    /// Returns `None` if the input is not `"1"`, `"2"`, `"3"`, or `"4"`.
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
