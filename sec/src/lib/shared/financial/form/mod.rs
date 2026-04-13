//! # Form Module
//!
//! Provides the [`Form`] enum representing SEC filing form types.
//! Each SEC filing is associated with a form type that determines the nature
//! and scope of the reported financial data.

use std::fmt;

/// SEC filing form type.
///
/// Represents the type of SEC filing from which a data point originates.
/// Uses `#[non_exhaustive]` to allow future extensions for additional form types.
///
/// # Example
/// ```
/// use sec::shared::financial::form::Form;
///
/// let form = Form::TenK;
/// assert_eq!(form.to_string(), "10-K");
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Form {
    /// Annual report (10-K).
    TenK,
    /// Quarterly report (10-Q).
    TenQ,
}

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TenK => write!(f, "10-K"),
            Self::TenQ => write!(f, "10-Q"),
        }
    }
}

impl Form {
    /// Attempts to parse a [`Form`] from an SEC form string.
    ///
    /// # Errors
    ///
    /// Returns `None` if the input string does not match any known form type.
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
