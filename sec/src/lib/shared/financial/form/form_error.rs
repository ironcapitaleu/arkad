//! # Form Errors
//!
//! Provides the [`UnsupportedForm`] error returned by [`Form::from_sec_str`](super::Form::from_sec_str).

use thiserror::Error;

/// Error indicating that an SEC form text is not one of the [`Form`](super::Form) variants or an
/// amendment of one.
#[derive(Debug, Error, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error(
    "[UnsupportedForm] SEC form '{form_text}' is not 10-K, 10-Q, 20-F, 40-F, or an amendment of one"
)]
pub struct UnsupportedForm {
    /// The form text that failed to parse.
    pub form_text: String,
}

impl UnsupportedForm {
    /// Creates a new [`UnsupportedForm`].
    pub fn new(form_text: impl Into<String>) -> Self {
        Self {
            form_text: form_text.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn should_format_display_with_form_text_when_form_is_unsupported() {
        let error = UnsupportedForm::new("8-K");

        let expected_result = "[UnsupportedForm] SEC form '8-K' is not 10-K, 10-Q, 20-F, 40-F, or an amendment of one";

        let result = error.to_string();

        assert_eq!(result, expected_result);
    }
}
