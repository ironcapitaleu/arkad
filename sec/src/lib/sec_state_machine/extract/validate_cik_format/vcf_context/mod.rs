use std::fmt;

use state_maschine::prelude::*;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// State context for the CIK format validation state.
///
/// The default instance uses the CIK for Berkshire Hathaway (CIK: 1067983).
pub struct ValidateCikFormatContext {
    /// The unvalidated CIK string provided for validation.
    pub raw_cik: String,
}

impl ValidateCikFormatContext {
    /// Creates a new instance of the state context for the CIK format validation.
    pub fn new(cik: &(impl ToString + ?Sized)) -> Self {
        Self {
            raw_cik: cik.to_string(),
        }
    }

    #[must_use]
    pub const fn cik(&self) -> &String {
        &self.raw_cik
    }
}

impl ContextData for ValidateCikFormatContext {
    type UpdateType = ValidateCikFormatContextUpdater;
    fn get_context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.raw_cik {
            self.raw_cik = cik;
        }
    }
}

const BERKSHIRE_HATHAWAY_CIK: &str = "1067983";

impl Default for ValidateCikFormatContext {
    /// Returns a default context using the CIK for Berkshire Hathaway (CIK: 1067983).
    fn default() -> Self {
        Self::new(BERKSHIRE_HATHAWAY_CIK)
    }
}

impl fmt::Display for ValidateCikFormatContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unvalidated CIK: {}", self.raw_cik)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
pub struct ValidateCikFormatContextUpdater {
    pub raw_cik: Option<String>,
}

/// Updater builder for the state context.
pub struct ValidateCikFormatContextUpdaterBuilder {
    raw_cik: Option<String>,
}
impl ValidateCikFormatContextUpdaterBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self { raw_cik: None }
    }

    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: String) -> Self {
        self.raw_cik = Some(cik);
        self
    }

    #[must_use]
    pub fn build(self) -> ValidateCikFormatContextUpdater {
        ValidateCikFormatContextUpdater {
            raw_cik: self.raw_cik,
        }
    }
}

impl Default for ValidateCikFormatContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use super::{ValidateCikFormatContext, ValidateCikFormatContextUpdaterBuilder};
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    #[test]
    fn should_return_reference_to_default_validation_context_when_initialized_with_default() {
        let validation_context = &ValidateCikFormatContext::default();

        let expected_result = &ValidateCikFormatContext::default();

        let result = validation_context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let validation_context = &ValidateCikFormatContext::new("0000000000");

        let default_validation_context = &ValidateCikFormatContext::default();

        let result = validation_context.get_context();

        assert_ne!(result, default_validation_context);
    }

    #[test]
    fn should_update_context_cik_data_to_specified_string_when_update_contains_specified_string() {
        let mut context = ValidateCikFormatContext::default();
        let update = ValidateCikFormatContextUpdaterBuilder::new()
            .cik("Updated CIK!".to_string())
            .build();

        let expected_result = &ValidateCikFormatContext::new("Updated CIK!");

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut context = ValidateCikFormatContext::default();
        let update = ValidateCikFormatContextUpdaterBuilder::new()
            .cik("First CIK Update!".to_string())
            .cik("Latest CIK Update!".to_string())
            .build();

        let expected_result = &ValidateCikFormatContext::new("Latest CIK Update!");

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_cik_data_the_default_when_update_contains_a_different_string() {
        let mut context = ValidateCikFormatContext::default();
        let update = ValidateCikFormatContextUpdaterBuilder::new()
            .cik("Updated CIK!".to_string())
            .build();

        context.update_context(update);
        let result = context.get_context().cik();

        assert_ne!(result, "1067983");
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = ValidateCikFormatContext::default();
        let empty_update = ValidateCikFormatContextUpdaterBuilder::default().build();

        let expected_result = &ValidateCikFormatContext::default();

        context.update_context(empty_update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }
}
