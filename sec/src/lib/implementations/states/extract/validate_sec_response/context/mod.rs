//! # Validate SEC Response Context Module
//!
//! This module defines the context data structures and updaters for the [`ValidateSecResponse`](../mod.rs) state in the SEC filings extraction workflow.
//!
//! The context provides stateful information required during the validation of an SEC response, such as retry configurations and CIK tracking. It is designed to be used with the [`ContextData`] trait, enabling ergonomic context management and updates within state machines.
//!
//! ## Components
//! - [`ValidateSecResponseContext`]: Holds the current context for validating an SEC response.
//! - [`ValidateSecResponseContextUpdater`]: Updater type for modifying context fields in a controlled way.
//! - [`ValidateSecResponseContextUpdaterBuilder`]: Builder for constructing context updaters with a fluent API.
//!
//! ## Usage
//! The context is used by the [`ValidateSecResponse`](../mod.rs) state to manage retry logic and track the CIK being processed. It supports updates via the builder pattern, making it easy to compose context changes in state machine workflows.
//!
//! ## Example
//! ```rust
//! use sec::implementations::states::extract::validate_sec_response::context::*;
//! use sec::shared::cik::Cik;
//! use state_maschine::prelude::*;
//!
//! let cik = Cik::new("1234567890").expect("Valid CIK");
//! let mut context = ValidateSecResponseContext::new(cik);
//! let update = ValidateSecResponseContextUpdaterBuilder::new()
//!     .cik(Cik::new("0987654321").expect("Valid CIK"))
//!     .build();
//! context.update_context(update);
//! assert_eq!(context.cik().to_string(), "0987654321");
//! ```
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::ContextData`]: Trait for context data management in states.
//! - [`crate::implementations::states::extract::validate_sec_response`]: Parent module for the SEC response validation state and data types.

use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;

use crate::shared::cik::Cik;
use crate::traits::state_machine::state::ContextData;

/// State context for the SEC response validation state.
///
/// This context holds configuration and tracking information required during the
/// validation of SEC HTTP responses, including the target CIK and retry settings.
/// It supports updates through the builder pattern and integrates with the
/// state machine framework's context management system.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateSecResponseContext {
    /// The Central Index Key (CIK) being processed in this validation context.
    pub cik: Cik,
    /// Maximum number of retry attempts allowed for failed validations.
    pub max_retries: u32,
}

impl ValidateSecResponseContext {
    /// Creates a new instance of the validate state context.
    ///
    /// # Arguments
    ///
    /// * `cik` - The [`Cik`] that will be tracked in this context.
    ///
    /// # Returns
    ///
    /// Returns a new [`ValidateSecResponseContext`] with the specified CIK and default retry settings.
    #[must_use]
    pub const fn new(cik: Cik) -> Self {
        Self {
            cik,
            max_retries: 0,
        }
    }

    /// Returns a reference to the context's CIK.
    ///
    /// # Returns
    ///
    /// A reference to the [`Cik`] being processed in this context.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }
}

impl ContextData for ValidateSecResponseContext {
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContextData for ValidateSecResponseContext {
    type UpdateType = ValidateSecResponseContextUpdater;

    fn get_context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.cik {
            self.cik = cik;
        }
    }
}

impl Default for ValidateSecResponseContext {
    fn default() -> Self {
        Self::new(Cik::default())
    }
}

impl fmt::Display for ValidateSecResponseContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tContext Data: {}", self.cik)
    }
}

/// Updater for modifying [`ValidateSecResponseContext`] in a controlled manner.
///
/// This struct allows for partial updates to context fields while maintaining
/// type safety and avoiding unnecessary allocations for unchanged fields.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateSecResponseContextUpdater {
    /// Optional new CIK to replace the current one.
    pub cik: Option<Cik>,
}

/// Builder for constructing [`ValidateSecResponseContextUpdater`] instances.
///
/// This builder provides a fluent API for constructing updaters with only
/// the fields that need to be changed, following the builder pattern.
pub struct ValidateSecResponseContextUpdaterBuilder {
    cik: Option<Cik>,
}

impl ValidateSecResponseContextUpdaterBuilder {
    /// Creates a new builder with no fields set to be updated.
    ///
    /// # Returns
    ///
    /// A new [`ValidateSecResponseContextUpdaterBuilder`] with all fields set to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self { cik: None }
    }

    /// Sets the CIK to be updated.
    ///
    /// # Arguments
    ///
    /// * `cik` - The new [`Cik`] to set in the context.
    ///
    /// # Returns
    ///
    /// The builder instance with the CIK field set for update.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    /// Builds the updater with the configured fields.
    ///
    /// # Returns
    ///
    /// A new [`ValidateSecResponseContextUpdater`] with the fields set by this builder.
    #[must_use]
    pub fn build(self) -> ValidateSecResponseContextUpdater {
        ValidateSecResponseContextUpdater { cik: self.cik }
    }
}

impl Default for ValidateSecResponseContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::ContextData as SMContextData;

    use super::*;
    use crate::shared::cik::Cik;

    #[test]
    fn should_create_new_context_with_provided_cik() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");

        let expected_result = (cik.clone(), 0);

        let result = ValidateSecResponseContext::new(cik);

        assert_eq!((result.cik().clone(), result.max_retries), expected_result);
    }

    #[test]
    fn should_return_cik_reference_when_accessing_cik() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let context = ValidateSecResponseContext::new(cik.clone());

        let expected_result = &cik;
        let result = context.cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_max_retries_when_accessing_max_retries() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let context = ValidateSecResponseContext::new(cik);

        let expected_result = 0;
        let result = context.get_max_retries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_when_updater_contains_cik() {
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid.");
        let mut context = ValidateSecResponseContext::new(original_cik);
        let updater = ValidateSecResponseContextUpdaterBuilder::new()
            .cik(new_cik.clone())
            .build();

        let expected_result = &ValidateSecResponseContext::new(new_cik);

        context.update_context(updater);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_update_fields_when_updater_is_empty() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let original_context = ValidateSecResponseContext::new(cik.clone());
        let mut context = original_context.clone();
        let updater = ValidateSecResponseContextUpdaterBuilder::new().build();

        let expected_result = &original_context;

        context.update_context(updater);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_default_context_when_default_is_called() {
        let expected_result = ValidateSecResponseContext {
            cik: Cik::default(),
            max_retries: 0,
        };

        let result = ValidateSecResponseContext::default();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_context_reference_when_accessing_context() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let context = ValidateSecResponseContext::new(cik);

        let expected_result = &context;
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_context_information_when_formatted() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let context = ValidateSecResponseContext::new(cik);

        let expected_result = "\tContext Data: 1234567890";
        let result = format!("{context}");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_to_latest_specified_value_when_multiple_updates_in_builder() {
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid.");
        let intermediate_cik =
            Cik::new("5555555555").expect("Hardcoded CIK should always be valid.");
        let final_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid.");
        let mut context = ValidateSecResponseContext::new(original_cik);
        let updater = ValidateSecResponseContextUpdaterBuilder::new()
            .cik(intermediate_cik)
            .cik(final_cik.clone())
            .build();

        let expected_result = &ValidateSecResponseContext::new(final_cik);

        context.update_context(updater);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_context_data_trait() {
        implements_auto_traits::<ValidateSecResponseContext>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_context_data_trait() {
        implements_send::<ValidateSecResponseContext>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_context_data_trait() {
        implements_sync::<ValidateSecResponseContext>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_context_data_trait() {
        implements_send::<ValidateSecResponseContext>();
        implements_sync::<ValidateSecResponseContext>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_context_data_trait() {
        implements_sized::<ValidateSecResponseContext>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_context_data_trait() {
        implements_hash::<ValidateSecResponseContext>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_context_data_trait() {
        implements_partial_eq::<ValidateSecResponseContext>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_context_data_trait() {
        implements_eq::<ValidateSecResponseContext>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_context_data_trait() {
        implements_partial_ord::<ValidateSecResponseContext>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_context_data_trait() {
        implements_ord::<ValidateSecResponseContext>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_context_data_trait() {
        implements_default::<ValidateSecResponseContext>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_context_data_trait() {
        implements_debug::<ValidateSecResponseContext>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_context_data_trait() {
        implements_clone::<ValidateSecResponseContext>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_context_data_trait() {
        implements_unpin::<ValidateSecResponseContext>();
    }
}
