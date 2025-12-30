//! # Execute SEC Request Context Module
//!
//! This module defines the context structures and updaters for the [`ExecuteSecRequest`](../mod.rs) state in the SEC filings extraction workflow.
//!
//! The context provides stateful information required during the execution of an SEC request, such as retry configurations and CIK tracking. It is designed to be used with the [`Context`] trait, enabling ergonomic context management and updates within state machines.
//!
//! ## Components
//! - [`ExecuteSecRequestContext`]: Holds the current context for executing an SEC request.
//! - [`ExecuteSecRequestContextUpdater`]: Updater type for modifying context fields in a controlled way.
//! - [`ExecuteSecRequestContextUpdaterBuilder`]: Builder for constructing context updaters with a fluent API.
//!
//! ## Usage
//! The context is used by the [`ExecuteSecRequest`](../mod.rs) state to manage retry logic and track the CIK being processed. It supports updates via the builder pattern, making it easy to compose context changes in state machine workflows.
//!
//! ## Example
//! ```rust
//! use sec::implementations::states::extract::execute_sec_request::context::*;
//! use sec::shared::cik::Cik;
//! use state_maschine::prelude::*;
//!
//! let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
//! let mut context = ExecuteSecRequestContext::new(cik);
//! let update = ExecuteSecRequestContextUpdater::builder()
//!     .cik(Cik::new("0987654321").expect("Hardcoded CIK should always be valid")
//!     .build();
//! context.update_context(update);
//! assert_eq!(context.cik().to_string(), "0987654321");
//! ```
//! ## See Also
//! - [`crate::traits::state_machine::state::Context`]: Trait for context management in states.
//! - [`crate::implementations::states::extract::execute_sec_request`]: Parent module for the SEC request execution state and data types.

use std::fmt;

use crate::shared::cik::Cik;
use crate::traits::state_machine::state::Context;

use state_maschine::prelude::Context as SMContext;

/// State context for the SEC request execution state.
///
/// This context holds configuration and tracking information required during the
/// execution of SEC HTTP requests, including the target CIK and retry settings.
/// It supports updates through the builder pattern and integrates with the
/// state machine framework's context management system.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestContext {
    /// The Central Index Key (CIK) being processed in this execution context.
    pub cik: Cik,
    /// Maximum number of retry attempts allowed for failed requests.
    pub max_retries: u32,
}

impl ExecuteSecRequestContext {
    /// Creates a new instance of the execute state context.
    ///
    /// # Arguments
    ///
    /// * `cik` - The [`Cik`] that will be tracked in this context.
    ///
    /// # Returns
    ///
    /// Returns a new [`ExecuteSecRequestContext`] with the specified CIK and default retry settings.
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

impl Context for ExecuteSecRequestContext {
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContext for ExecuteSecRequestContext {
    type UpdateType = ExecuteSecRequestContextUpdater;

    fn context(&self) -> &Self {
        self
    }

    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.cik {
            self.cik = cik;
        }
    }
}

impl Default for ExecuteSecRequestContext {
    fn default() -> Self {
        Self::new(Cik::default())
    }
}

impl fmt::Display for ExecuteSecRequestContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Context Data: {}", self.cik)
    }
}

/// Updater for modifying [`ExecuteSecRequestContext`] in a controlled manner.
///
/// This struct allows for partial updates to context fields while maintaining
/// type safety and avoiding unnecessary allocations for unchanged fields.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequestContextUpdater {
    /// Optional new CIK to replace the current one.
    pub cik: Option<Cik>,
}

impl ExecuteSecRequestContextUpdater {
    /// Creates a new builder for constructing [`ExecuteSecRequestContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ExecuteSecRequestContextUpdaterBuilder {
        ExecuteSecRequestContextUpdaterBuilder::new()
    }
}

/// Builder for constructing [`ExecuteSecRequestContextUpdater`] instances.
///
/// This builder provides a fluent API for constructing updaters with only
/// the fields that need to be changed, following the builder pattern.
pub struct ExecuteSecRequestContextUpdaterBuilder {
    cik: Option<Cik>,
}

impl ExecuteSecRequestContextUpdaterBuilder {
    /// Creates a new builder with no fields set to be updated.
    ///
    /// # Returns
    ///
    /// A new [`ExecuteSecRequestContextUpdaterBuilder`] with all fields set to `None`.
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
    /// A new [`ExecuteSecRequestContextUpdater`] with the fields set by this builder.
    #[must_use]
    pub fn build(self) -> ExecuteSecRequestContextUpdater {
        ExecuteSecRequestContextUpdater { cik: self.cik }
    }
}

impl Default for ExecuteSecRequestContextUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::cik::Cik;
    use pretty_assertions::assert_eq;
    use state_maschine::prelude::Context as SMContext;
    use std::{fmt::Debug, hash::Hash};

    #[test]
    fn should_create_new_context_with_provided_cik() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let expected_cik = cik.clone();
        let expected_max_retries = 0;

        let result = ExecuteSecRequestContext::new(cik);

        assert_eq!(result.cik(), &expected_cik);
        assert_eq!(result.max_retries, expected_max_retries);
    }

    #[test]
    fn should_return_cik_reference_when_accessing_cik() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = ExecuteSecRequestContext::new(cik.clone());

        let expected_result = &cik;
        let result = context.cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_max_retries_when_accessing_max_retries() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = ExecuteSecRequestContext::new(cik);

        let expected_result = 0;
        let result = context.max_retries();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_when_updater_contains_cik() {
        let original_cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let new_cik = Cik::new("0987654321").expect("Hardcoded CIK should always be valid");
        let mut context = ExecuteSecRequestContext::new(original_cik);

        let updater = ExecuteSecRequestContextUpdater::builder()
            .cik(new_cik.clone())
            .build();

        context.update_context(updater);

        assert_eq!(context.cik(), &new_cik);
    }

    #[test]
    fn should_not_update_fields_when_updater_is_empty() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let original_context = ExecuteSecRequestContext::new(cik.clone());
        let mut context = original_context.clone();

        let updater = ExecuteSecRequestContextUpdater::builder().build();

        context.update_context(updater);

        assert_eq!(context, original_context);
    }

    #[test]
    fn should_create_default_context_when_default_is_called() {
        let expected_result = ExecuteSecRequestContext {
            cik: Cik::default(),
            max_retries: 0,
        };

        let result = ExecuteSecRequestContext::default();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_context_reference_when_accessing_context() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = ExecuteSecRequestContext::new(cik);

        let expected_result = &context;
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_context_information_when_formatted() {
        let cik = Cik::new("1234567890").expect("Hardcoded CIK should always be valid");
        let context = ExecuteSecRequestContext::new(cik);

        let expected_result = "Context Data: 1234567890";
        let result = format!("{context}");

        assert_eq!(result, expected_result);
    }

    // Trait implementation tests
    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_context_data_trait() {
        implements_auto_traits::<ExecuteSecRequestContext>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_context_data_trait() {
        implements_send::<ExecuteSecRequestContext>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_context_data_trait() {
        implements_sync::<ExecuteSecRequestContext>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_context_data_trait() {
        implements_send::<ExecuteSecRequestContext>();
        implements_sync::<ExecuteSecRequestContext>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_context_data_trait() {
        implements_sized::<ExecuteSecRequestContext>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_context_data_trait() {
        implements_hash::<ExecuteSecRequestContext>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_context_data_trait() {
        implements_partial_eq::<ExecuteSecRequestContext>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_context_data_trait() {
        implements_eq::<ExecuteSecRequestContext>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_context_data_trait() {
        implements_partial_ord::<ExecuteSecRequestContext>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_context_data_trait() {
        implements_ord::<ExecuteSecRequestContext>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_context_data_trait() {
        implements_default::<ExecuteSecRequestContext>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_context_data_trait() {
        implements_debug::<ExecuteSecRequestContext>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_context_data_trait() {
        implements_clone::<ExecuteSecRequestContext>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_context_data_trait() {
        implements_unpin::<ExecuteSecRequestContext>();
    }
}
