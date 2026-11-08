//! # Fiscal Period
//!
//! Provides [`FiscalPeriod`], the fiscal period of a filing as the filer reports it.

use std::fmt;

use super::invalid_value::{InvalidValue, require_token};

/// The fiscal period of a filing as filed, for example `FY`, `Q1`, or `H1`.
///
/// The set of periods is open. An SEC filer reports `FY` and `Q1` to `Q4`. An ESEF filer reports
/// half years.
///
/// # Examples
///
/// ```
/// use xbrl::core::fiscal_period::FiscalPeriod;
///
/// let text = "H1";
///
/// let expected_result = "H1";
///
/// let period = FiscalPeriod::new(text).expect("Given a hardcoded period without whitespace, the creation should always succeed");
/// let result = period.as_str();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct FiscalPeriod(String);

impl FiscalPeriod {
    /// Creates a [`FiscalPeriod`] from its text.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if the text is empty or holds whitespace.
    pub fn new(text: impl Into<String>) -> Result<Self, InvalidValue> {
        let text = text.into();
        require_token("FiscalPeriod", &text)?;
        Ok(Self(text))
    }

    /// Returns the text of the fiscal period.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FiscalPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::invalid_value::InvalidValueReason;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_fiscal_period() {
        implements_auto_traits::<FiscalPeriod>();
    }

    #[test]
    fn should_reject_period_when_text_is_empty() {
        let text = "";

        let expected_result = Err(InvalidValue::new(
            "FiscalPeriod",
            InvalidValueReason::Empty,
            "",
        ));

        let result = FiscalPeriod::new(text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_print_text_when_displayed() {
        let period = FiscalPeriod::new("FY").expect(
            "Given a hardcoded period without whitespace, the creation should always succeed",
        );

        let expected_result = "FY";

        let result = period.to_string();

        assert_eq!(result, expected_result);
    }
}
