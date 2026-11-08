//! # Form
//!
//! Provides [`Form`], the form type of a filing as the filer files it.

use std::fmt;

use super::invalid_value::{InvalidValue, InvalidValueReason};

/// The form type of a filing as filed, for example `10-K`, `10-Q/A`, `20-F`, or `AFR`.
///
/// The set of forms is open, because each regulator has its own forms. Only an empty text is
/// invalid.
///
/// # Examples
///
/// ```
/// use xbrl::core::form::Form;
///
/// let form = Form::new("10-K/A").expect("Given a hardcoded form type, the creation should always succeed");
///
/// let expected_result = true;
///
/// let result = form.is_amendment();
///
/// assert_eq!(result, expected_result);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct Form(String);

impl Form {
    /// The type name that [`InvalidValue`] names when an input breaks a rule.
    const TYPE_NAME: &'static str = "Form";

    /// The suffix that marks an SEC amendment, as in `10-K/A`.
    pub const AMENDMENT_SUFFIX: &'static str = "/A";

    /// Creates a [`Form`] from its text.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] with [`InvalidValueReason::Empty`] if the text is empty.
    pub fn new(text: impl Into<String>) -> Result<Self, InvalidValue> {
        let text = text.into();
        if text.is_empty() {
            return Err(InvalidValue::new(
                Self::TYPE_NAME,
                InvalidValueReason::Empty,
                text,
            ));
        }
        Ok(Self(text))
    }

    /// Returns the text of the form type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns `true` if the form type ends with `/A`, the mark of an SEC amendment.
    ///
    /// A source that marks amendments in another way sets `Filing::is_amendment` by its own rule.
    #[must_use]
    pub fn is_amendment(&self) -> bool {
        self.0.ends_with(Self::AMENDMENT_SUFFIX)
    }
}

impl fmt::Display for Form {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_form() {
        implements_auto_traits::<Form>();
    }

    #[test]
    fn should_keep_text_when_form_is_not_an_sec_form() {
        let text = "20-F";

        let expected_result = "20-F";

        let result = Form::new(text)
            .expect("Given a hardcoded form type, the creation should always succeed")
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_an_amendment_when_form_has_no_suffix() {
        let form = Form::new("10-Q")
            .expect("Given a hardcoded form type, the creation should always succeed");

        let expected_result = false;

        let result = form.is_amendment();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_reject_form_when_text_is_empty() {
        let text = "";

        let expected_result = Err(InvalidValue::new("Form", InvalidValueReason::Empty, ""));

        let result = Form::new(text);

        assert_eq!(result, expected_result);
    }
}
