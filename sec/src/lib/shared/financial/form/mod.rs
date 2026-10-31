//! # Form
//!
//! Provides the [`Form`] enum representing the SEC filing types a data point can originate from,
//! the [`SELECTED_FORMS`] that both ingestion paths keep, and the [`ParsedForm`] that
//! [`Form::from_sec_str`] returns.
//!
//! ## Modules
//!
//! - [`form_error`]: The [`UnsupportedForm`] error returned when the form text has no variant.

use std::fmt::{self, Display, Formatter};

use serde::Serialize;

pub mod form_error;

pub use form_error::UnsupportedForm;

/// The forms that both ingestion paths keep.
pub const SELECTED_FORMS: [Form; 4] = [Form::TenK, Form::TenQ, Form::TwentyF, Form::FortyF];

/// The suffix the SEC appends to the form text of an amendment, as in `10-K/A`.
const AMENDMENT_SUFFIX: &str = "/A";

/// The SEC filing form a data point originates from.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Form {
    /// Annual report (10-K).
    TenK,
    /// Quarterly report (10-Q).
    TenQ,
    /// Annual report of a foreign private issuer (20-F).
    TwentyF,
    /// Annual report of a Canadian issuer under the multijurisdictional disclosure system (40-F).
    FortyF,
}

impl Display for Form {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::TenK => write!(f, "10-K"),
            Self::TenQ => write!(f, "10-Q"),
            Self::TwentyF => write!(f, "20-F"),
            Self::FortyF => write!(f, "40-F"),
        }
    }
}

impl Form {
    /// Parses an SEC form text such as `"10-K"` or `"10-K/A"` into a [`ParsedForm`].
    ///
    /// A text with the `/A` suffix gives its base form, with [`ParsedForm::is_amendment`] true.
    /// A text with leading or trailing whitespace returns an [`UnsupportedForm`], so the caller must
    /// trim it.
    ///
    /// # Errors
    ///
    /// Returns an [`UnsupportedForm`] if the text, without `/A`, is not the text of a [`Form`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sec::shared::financial::form::{Form, ParsedForm};
    ///
    /// let expected_result = Ok(ParsedForm::new(Form::TenK, true));
    ///
    /// let result = Form::from_sec_str("10-K/A");
    ///
    /// assert_eq!(result, expected_result);
    /// ```
    pub fn from_sec_str(s: &str) -> Result<ParsedForm, UnsupportedForm> {
        let (base, is_amendment) = s
            .strip_suffix(AMENDMENT_SUFFIX)
            .map_or((s, false), |base| (base, true));
        let form = match base {
            "10-K" => Self::TenK,
            "10-Q" => Self::TenQ,
            "20-F" => Self::TwentyF,
            "40-F" => Self::FortyF,
            _ => return Err(UnsupportedForm::new(s)),
        };

        Ok(ParsedForm::new(form, is_amendment))
    }
}

/// A [`Form`] parsed from an SEC form text, and whether the filing amends an earlier one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ParsedForm {
    form: Form,
    is_amendment: bool,
}

impl ParsedForm {
    /// Creates a new [`ParsedForm`].
    #[must_use]
    pub const fn new(form: Form, is_amendment: bool) -> Self {
        Self { form, is_amendment }
    }

    /// Returns the base form, for example [`Form::TenK`] for `10-K/A`.
    #[must_use]
    pub const fn form(&self) -> Form {
        self.form
    }

    /// Returns `true` if the form text had the `/A` suffix.
    #[must_use]
    pub const fn is_amendment(&self) -> bool {
        self.is_amendment
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_display_10k_when_form_is_ten_k() {
        let form = Form::TenK;

        let expected_result = "10-K";

        let result = form.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_10q_when_form_is_ten_q() {
        let form = Form::TenQ;

        let expected_result = "10-Q";

        let result = form.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_20f_when_form_is_twenty_f() {
        let form = Form::TwentyF;

        let expected_result = "20-F";

        let result = form.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_40f_when_form_is_forty_f() {
        let form = Form::FortyF;

        let expected_result = "40-F";

        let result = form.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_ten_k_from_sec_string_when_input_is_10k() {
        let sec_string = "10-K";

        let expected_result = Ok(ParsedForm::new(Form::TenK, false));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_ten_q_from_sec_string_when_input_is_10q() {
        let sec_string = "10-Q";

        let expected_result = Ok(ParsedForm::new(Form::TenQ, false));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_twenty_f_from_sec_string_when_input_is_20f() {
        let sec_string = "20-F";

        let expected_result = Ok(ParsedForm::new(Form::TwentyF, false));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_forty_f_from_sec_string_when_input_is_40f() {
        let sec_string = "40-F";

        let expected_result = Ok(ParsedForm::new(Form::FortyF, false));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_ten_k_amendment_from_sec_string_when_input_is_10k_a() {
        let sec_string = "10-K/A";

        let expected_result = Ok(ParsedForm::new(Form::TenK, true));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_ten_q_amendment_from_sec_string_when_input_is_10q_a() {
        let sec_string = "10-Q/A";

        let expected_result = Ok(ParsedForm::new(Form::TenQ, true));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_twenty_f_amendment_from_sec_string_when_input_is_20f_a() {
        let sec_string = "20-F/A";

        let expected_result = Ok(ParsedForm::new(Form::TwentyF, true));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_forty_f_amendment_from_sec_string_when_input_is_40f_a() {
        let sec_string = "40-F/A";

        let expected_result = Ok(ParsedForm::new(Form::FortyF, true));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unsupported_form_when_sec_string_is_unknown_form() {
        let sec_string = "8-K";

        let expected_result = Err(UnsupportedForm::new(sec_string));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unsupported_form_when_sec_string_is_amendment_of_unknown_form() {
        let sec_string = "8-K/A";

        let expected_result = Err(UnsupportedForm::new(sec_string));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unsupported_form_when_sec_string_is_bare_amendment_suffix() {
        let sec_string = "/A";

        let expected_result = Err(UnsupportedForm::new(sec_string));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unsupported_form_when_sec_string_has_amendment_suffix_twice() {
        let sec_string = "10-K/A/A";

        let expected_result = Err(UnsupportedForm::new(sec_string));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unsupported_form_when_sec_string_has_trailing_whitespace() {
        let form_text = "10-K   ";

        let expected_result = Err(UnsupportedForm::new(form_text));

        let result = Form::from_sec_str(form_text);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unsupported_form_when_sec_string_is_lowercase() {
        let sec_string = "10-k";

        let expected_result = Err(UnsupportedForm::new(sec_string));

        let result = Form::from_sec_str(sec_string);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_parse_each_displayed_form_back_to_itself_for_selected_forms() {
        let expected_result: Vec<Result<ParsedForm, UnsupportedForm>> = SELECTED_FORMS
            .iter()
            .map(|form| Ok(ParsedForm::new(*form, false)))
            .collect();

        let result: Vec<Result<ParsedForm, UnsupportedForm>> = SELECTED_FORMS
            .iter()
            .map(|form| Form::from_sec_str(&form.to_string()))
            .collect();

        assert_eq!(result, expected_result);
    }
}
