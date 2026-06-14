//! # Validate CIK Format Context
//!
//! Provides the [`ValidateCikFormatContext`] carried alongside the
//! [`ValidateCikFormat`](crate::implementations::states::extract::validate_cik_format::ValidateCikFormat)
//! state, together with its updater and builder.
//!
//! The context holds ambient information that outlives any single input/output pair —
//! the shared [`SecClient`] and the retry budget — so the state can be re-run and
//! transitioned without reconstructing those resources. Updates are applied through the
//! [`Context`] trait via a builder-constructed updater, keeping mutation explicit and partial.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::validate_cik_format::context::*;
//! use sec::shared::http_client::implementations::sec_client::SecClient;
//! use state_maschine::prelude::*;
//!
//! let mut context = ValidateCikFormatContext::new("1067983", SecClient::default());
//! let update = ValidateCikFormatContextUpdater::builder()
//!     .cik("0000000001")
//!     .build();
//!
//! context.update_context(update);
//! assert_eq!(context.cik(), "0000000001");
//! ```
//!
//! ## See Also
//!
//! - [`crate::traits::state_machine::state::Context`]: Trait that defines context updates and the retry budget.
//! - [`crate::implementations::states::extract::validate_cik_format`]: Parent module for the validation state and its data types.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;

use crate::shared::http_client::implementations::sec_client::SecClient;
use crate::traits::state_machine::state::Context;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
/// Ambient context for the [`ValidateCikFormat`](super::ValidateCikFormat) state.
///
/// Bundles the resources and configuration that persist across the state's lifetime —
/// the raw CIK under validation, the shared [`SecClient`], and the retry budget — so they
/// survive updates and transitions.
pub struct ValidateCikFormatContext {
    /// The unvalidated CIK string provided for validation.
    pub raw_cik: String,
    /// The shared HTTP client for SEC API requests, passed down from the super-state.
    pub sec_client: SecClient,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl ValidateCikFormatContext {
    /// Creates a new context from a raw CIK and a shared HTTP client.
    ///
    /// # Arguments
    ///
    /// * `cik` - The raw CIK to validate, as any value convertible into a [`String`].
    /// * `sec_client` - The shared HTTP client for SEC API requests.
    pub fn new(cik: impl Into<String>, sec_client: SecClient) -> Self {
        Self {
            raw_cik: cik.into(),
            sec_client,
            max_retries: 0,
        }
    }

    /// Returns a reference to the current raw CIK string in the context.
    #[must_use]
    pub const fn cik(&self) -> &String {
        &self.raw_cik
    }

    /// Returns a reference to the HTTP client.
    #[must_use]
    pub const fn sec_client(&self) -> &SecClient {
        &self.sec_client
    }
}

impl Context for ValidateCikFormatContext {
    /// Returns the maximum number of retries allowed for CIK validation.
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContext for ValidateCikFormatContext {
    type UpdateType = ValidateCikFormatContextUpdater;

    /// Returns a reference to the current context.
    fn context(&self) -> &Self {
        self
    }

    /// Updates the context fields using the provided updater.
    ///
    /// Only fields set in the updater will be changed.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.raw_cik {
            self.raw_cik = cik;
        }
        if let Some(sec_client) = updates.sec_client {
            self.sec_client = sec_client;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl fmt::Display for ValidateCikFormatContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unvalidated CIK: {}", self.raw_cik)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Partial update for a [`ValidateCikFormatContext`].
///
/// Each field is optional; only the fields set to `Some` are applied when the updater is
/// passed to [`update_context`](state_maschine::state_machine::state::Context::update_context).
pub struct ValidateCikFormatContextUpdater {
    /// Optional new raw CIK string value.
    pub raw_cik: Option<String>,
    /// Optional new HTTP client value.
    pub sec_client: Option<SecClient>,
    /// Optional new maximum retries value.
    pub max_retries: Option<u32>,
}

impl ValidateCikFormatContextUpdater {
    /// Creates a new builder for constructing [`ValidateCikFormatContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ValidateCikFormatContextUpdaterBuilder {
        ValidateCikFormatContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`ValidateCikFormatContextUpdater`].
pub struct ValidateCikFormatContextUpdaterBuilder {
    raw_cik: Option<String>,
    sec_client: Option<SecClient>,
    max_retries: Option<u32>,
}
impl ValidateCikFormatContextUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            raw_cik: None,
            sec_client: None,
            max_retries: None,
        }
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

    /// Sets the HTTP client value inside the context to the provided update value.
    ///
    /// # Arguments
    /// * `sec_client` - The new HTTP client.
    #[must_use]
    pub fn sec_client(mut self, sec_client: SecClient) -> Self {
        self.sec_client = Some(sec_client);
        self
    }

    /// Sets the `max_retries` value inside the context to the provided update value.
    ///
    /// # Arguments
    /// * `max_retries` - The new value for `max_retries`.
    #[must_use]
    pub const fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the updater with the specified fields.
    #[must_use]
    pub fn build(self) -> ValidateCikFormatContextUpdater {
        ValidateCikFormatContextUpdater {
            raw_cik: self.raw_cik,
            sec_client: self.sec_client,
            max_retries: self.max_retries,
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
        ValidateCikFormatContext, ValidateCikFormatContextUpdater,
        ValidateCikFormatContextUpdaterBuilder,
    };
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::http_client::implementations::sec_client::SecClient;

    fn test_context() -> ValidateCikFormatContext {
        let sec_client = SecClient::default();
        ValidateCikFormatContext::new(BERKSHIRE_HATHAWAY_CIK_RAW, sec_client)
    }

    #[test]
    fn should_return_reference_to_default_validation_context_when_initialized_with_default() {
        let validation_context = test_context();

        let expected_result = &test_context();

        let result = validation_context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let sec_client = SecClient::default();
        let validation_context = &ValidateCikFormatContext::new("0000000000", sec_client);

        let expected_result = &test_context();

        let result = validation_context.context();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_update_context_cik_data_to_specified_string_when_update_contains_specified_string() {
        let mut context = test_context();
        let update = ValidateCikFormatContextUpdater::builder()
            .cik("Updated CIK!")
            .build();

        let sec_client = SecClient::default();
        let expected_result = &ValidateCikFormatContext::new("Updated CIK!", sec_client);

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_to_latest_specified_string_when_multiple_updates_in_builder() {
        let mut context = test_context();
        let update = ValidateCikFormatContextUpdater::builder()
            .cik("First CIK Update!")
            .cik("Latest CIK Update!")
            .build();

        let sec_client = SecClient::default();
        let expected_result = &ValidateCikFormatContext::new("Latest CIK Update!", sec_client);

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_cik_data_the_default_when_update_contains_a_different_string() {
        let mut context = test_context();
        let update = ValidateCikFormatContextUpdater::builder()
            .cik("Updated CIK!")
            .build();

        let expected_result = BERKSHIRE_HATHAWAY_CIK_RAW;

        context.update_context(update);
        let result = context.context().cik();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = test_context();
        let empty_update = ValidateCikFormatContextUpdaterBuilder::default().build();

        let expected_result = &test_context();

        context.update_context(empty_update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_updater_contains_max_retries() {
        let mut context = test_context();
        let update = ValidateCikFormatContextUpdater::builder()
            .max_retries(5)
            .build();

        let expected_result = 5;

        context.update_context(update);
        let result = context.max_retries;

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
