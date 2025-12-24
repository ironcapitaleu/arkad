//! # Validate CIK Format Context Module
//!
//! This module defines the context data structures and updaters for the [`ValidateCikFormat`](../mod.rs) state in the SEC filings extraction workflow.
//!
//! The context provides stateful information required during CIK format validation, such as the raw CIK string and retry configuration. It is designed to be used with the [`ContextData`] trait, enabling ergonomic context management and updates within state machines.
//!
//! ## Components
//! - [`ValidateCikFormatContext`]: Holds the current context for CIK validation, including the raw CIK and retry count.
//! - [`ValidateCikFormatContextUpdater`]: Updater type for modifying context fields in a controlled way.
//! - [`ValidateCikFormatContextUpdaterBuilder`]: Builder for constructing context updaters with a fluent API.
//!
//! ## Usage
//! The context is used by the [`ValidateCikFormat`](../mod.rs) state to track the CIK being validated and manage retry logic. It supports updates via the builder pattern, making it easy to compose context changes in state machine workflows.
//!
//! ## Example
//! ```rust
//! use sec::implementations::states::extract::validate_cik_format::context::*;
//! use state_maschine::prelude::*;
//!
//! let mut context = ValidateCikFormatContext::default();
//! let update = ValidateCikFormatContextUpdater::builder()
//!     .cik("0000000001")
//!     .build();
//! context.update_context(update);
//! assert_eq!(context.cik(), "0000000001");
//! ```
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::ContextData`]: Trait for context data management in states.
//! - [`crate::implementations::states::extract::validate_cik_format`]: Parent module for CIK validation state and data types.

use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;

use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
use crate::traits::state_machine::state::ContextData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// State context for the CIK format validation state.
///
/// The default instance uses the CIK for Berkshire Hathaway (CIK: 1067983).
pub struct ValidateCikFormatContext {
    /// The unvalidated CIK string provided for validation.
    pub raw_cik: String,
    pub max_retries: u32,
}

impl ValidateCikFormatContext {
    /// Creates a new instance of the state context for the CIK format validation.
    ///
    /// # Arguments
    /// * `raw_cik` - A value that can be converted to a string, representing the raw CIK to validate.
    ///
    /// # Returns
    /// A new `ValidateCikFormatContext` with the provided CIK and default retry count.
    pub fn new(cik: impl Into<String>) -> Self {
        Self {
            raw_cik: cik.into(),
            max_retries: 0,
        }
    }

    /// Returns a reference to the current raw CIK string in the context.
    #[must_use]
    pub const fn cik(&self) -> &String {
        &self.raw_cik
    }
}

impl ContextData for ValidateCikFormatContext {
    /// Returns the maximum number of retries allowed for CIK validation.
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContextData for ValidateCikFormatContext {
    type UpdateType = ValidateCikFormatContextUpdater;

    /// Returns a reference to the current context.
    fn get_context(&self) -> &Self {
        self
    }

    /// Updates the context fields using the provided updater.
    ///
    /// Only fields set in the updater will be changed.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.raw_cik {
            self.raw_cik = cik;
        }
    }
}

impl Default for ValidateCikFormatContext {
    /// Returns a default context using the CIK for Berkshire Hathaway (CIK: 1067983).
    fn default() -> Self {
        Self::new(BERKSHIRE_HATHAWAY_CIK_RAW)
    }
}

impl fmt::Display for ValidateCikFormatContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unvalidated CIK: {}", self.raw_cik)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
///
/// Using this struct allows to update fields of `ValidateCikFormatContext` in a controlled way.
pub struct ValidateCikFormatContextUpdater {
    pub raw_cik: Option<String>,
}

impl ValidateCikFormatContextUpdater {
    /// Creates a new builder for constructing [`ValidateCikFormatContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ValidateCikFormatContextUpdaterBuilder {
        ValidateCikFormatContextUpdaterBuilder::new()
    }
}

/// Builder for `ValidateCikFormatContextUpdater`.
///
/// Use this builder to fluently construct an updater for the context.
pub struct ValidateCikFormatContextUpdaterBuilder {
    raw_cik: Option<String>,
}
impl ValidateCikFormatContextUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self { raw_cik: None }
    }

    /// Sets the raw CIK value inside the context to the provided update value.
    ///
    /// # Arguments
    /// * `cik` - A value that can be converted to a string, representing the new raw CIK.    
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: impl Into<String>) -> Self {
        self.raw_cik = Some(cik.into());
        self
    }

    /// Builds the updater with the specified fields.
    #[must_use]
    pub fn build(self) -> ValidateCikFormatContextUpdater {
        ValidateCikFormatContextUpdater {
            raw_cik: self.raw_cik,
        }
    }
}

impl Default for ValidateCikFormatContextUpdaterBuilder {
    /// Returns a new context update builder with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    use super::{
        BERKSHIRE_HATHAWAY_CIK_RAW, ValidateCikFormatContext, ValidateCikFormatContextUpdater,
        ValidateCikFormatContextUpdaterBuilder,
    };

    #[test]
    fn should_return_reference_to_default_validation_context_when_initialized_with_default() {
        let validation_context = ValidateCikFormatContext::default();

        let expected_result = &ValidateCikFormatContext::default();

        let result = validation_context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let validation_context = &ValidateCikFormatContext::new("0000000000");

        let expected_result = &ValidateCikFormatContext::default();

        let result = validation_context.get_context();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_update_context_cik_data_to_specified_string_when_update_contains_specified_string() {
        let mut context = ValidateCikFormatContext::default();
        let update = ValidateCikFormatContextUpdater::builder()
            .cik("Updated CIK!")
            .build();

        let expected_result = &ValidateCikFormatContext::new("Updated CIK!");

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut context = ValidateCikFormatContext::default();
        let update = ValidateCikFormatContextUpdater::builder()
            .cik("First CIK Update!")
            .cik("Latest CIK Update!")
            .build();

        let expected_result = &ValidateCikFormatContext::new("Latest CIK Update!");

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_cik_data_the_default_when_update_contains_a_different_string() {
        let mut context = ValidateCikFormatContext::default();
        let update = ValidateCikFormatContextUpdater::builder()
            .cik("Updated CIK!")
            .build();

        let expected_result = BERKSHIRE_HATHAWAY_CIK_RAW;

        context.update_context(update);
        let result = context.get_context().cik();

        assert_ne!(result, expected_result);
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

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_context_data_trait() {
        implements_auto_traits::<ValidateCikFormatContext>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_context_data_trait() {
        implements_send::<ValidateCikFormatContext>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_context_data_trait() {
        implements_sync::<ValidateCikFormatContext>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_context_data_trait() {
        implements_send::<ValidateCikFormatContext>();
        implements_sync::<ValidateCikFormatContext>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_context_data_trait() {
        implements_sized::<ValidateCikFormatContext>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_context_data_trait() {
        implements_hash::<ValidateCikFormatContext>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_context_data_trait() {
        implements_partial_eq::<ValidateCikFormatContext>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_context_data_trait() {
        implements_eq::<ValidateCikFormatContext>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_context_data_trait() {
        implements_partial_ord::<ValidateCikFormatContext>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_context_data_trait() {
        implements_ord::<ValidateCikFormatContext>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_context_data_trait() {
        implements_default::<ValidateCikFormatContext>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_context_data_trait() {
        implements_debug::<ValidateCikFormatContext>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_context_data_trait() {
        implements_clone::<ValidateCikFormatContext>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_context_data_trait() {
        implements_unpin::<ValidateCikFormatContext>();
    }
}
