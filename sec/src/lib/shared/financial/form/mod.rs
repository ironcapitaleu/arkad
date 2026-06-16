//! # Form
//!
//! Provides the [`Form`] enum representing the SEC filing types a data point can originate from.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

/// The SEC filing form a data point originates from.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Form {
    /// Annual report (10-K).
    TenK,
    /// Quarterly report (10-Q).
    TenQ,
}

impl Display for Form {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::TenK => write!(f, "10-K"),
            Self::TenQ => write!(f, "10-Q"),
        }
    }
}

impl Form {
    /// Parses a [`Form`] from its SEC form string (e.g. `"10-K"`), or `None` if unrecognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::financial::form::Form;
    ///
    /// assert_eq!(Form::from_sec_str("10-K"), Some(Form::TenK));
    /// assert_eq!(Form::from_sec_str("8-K"), None);
    /// ```
    #[must_use]
    pub fn from_sec_str(s: &str) -> Option<Self> {
        match s {
            "10-K" => Some(Self::TenK),
            "10-Q" => Some(Self::TenQ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_display_10k_when_form_is_ten_k() {
        let expected_result = "10-K";

        let result = Form::TenK.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_ten_k_from_sec_string_when_input_is_10k() {
        let expected_result = Some(Form::TenK);

        let result = Form::from_sec_str("10-K");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_none_when_sec_string_is_unknown_form() {
        let expected_result = None;

        let result = Form::from_sec_str("8-K");

        assert_eq!(result, expected_result);
    }
}
