//! # Prepare SEC Request Context Module
//!
//! This module defines the context data structures and updaters for the [`PrepareSecRequest`](../mod.rs) state in the SEC filings extraction workflow.
//!
//! The context provides stateful information required during the preparation of an SEC request, such as retry configurations. It is designed to be used with the [`ContextData`] trait, enabling ergonomic context management and updates within state machines.
//!
//! ## Components
//! - [`PrepareSecRequestContext`]: Holds the current context for preparing an SEC request.
//! - [`PrepareSecRequestContextUpdater`]: Updater type for modifying context fields in a controlled way.
//! - [`PrepareSecRequestContextUpdaterBuilder`]: Builder for constructing context updaters with a fluent API.
//!
//! ## Usage
//! The context is used by the [`PrepareSecRequest`](../mod.rs) state to manage retry logic. It supports updates via the builder pattern, making it easy to compose context changes in state machine workflows.
//!
//! ## Example
//! ```rust
//! use sec::implementations::states::extract::prepare_sec_request::context::*;
//! use state_maschine::prelude::*;
//!
//! let mut context = PrepareSecRequestContext::default();
//! let update = PrepareSecRequestContextUpdater::builder()
//!     .max_retries(5)
//!     .build();
//! context.update_context(update);
//! assert_eq!(context.max_retries, 5);
//! ```
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::ContextData`]: Trait for context data management in states.
//! - [`crate::implementations::states::extract::prepare_sec_request`]: Parent module for the SEC request preparation state and data types.

use std::fmt;

use state_maschine::prelude::ContextData as SMContextData;

use crate::shared::cik::Cik;
use crate::traits::state_machine::state::ContextData;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// State context for the SEC request preparation state.
pub struct PrepareSecRequestContext {
    pub cik: Cik,
    pub max_retries: u32,
}

impl PrepareSecRequestContext {
    #[must_use]
    /// Creates a new instance of the state context for SEC request preparation.
    /// # Arguments
    /// * `cik` - The Central Index Key (CIK) associated with the SEC request.
    pub const fn new(cik: Cik) -> Self {
        Self {
            cik,
            max_retries: 0,
        }
    }
}

impl ContextData for PrepareSecRequestContext {
    /// Returns the maximum number of retries allowed for the SEC request.
    fn get_max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContextData for PrepareSecRequestContext {
    type UpdateType = PrepareSecRequestContextUpdater;

    /// Returns a reference to the current context.
    fn get_context(&self) -> &Self {
        self
    }

    /// Updates the context fields using the provided updater.
    ///
    /// Only fields set in the updater will be changed.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }

        if let Some(cik) = updates.cik {
            self.cik = cik;
        }
    }
}

impl fmt::Display for PrepareSecRequestContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Context Data:\nCIK (validated): {}\nMax retries: {}",
            self.cik, self.max_retries
        )
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
///
/// Using this struct allows you to update fields of [`PrepareSecRequestContext`] in a controlled way.
pub struct PrepareSecRequestContextUpdater {
    pub cik: Option<Cik>,
    pub max_retries: Option<u32>,
}

impl PrepareSecRequestContextUpdater {
    /// Creates a new builder for constructing [`PrepareSecRequestContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> PrepareSecRequestContextUpdaterBuilder {
        PrepareSecRequestContextUpdaterBuilder::new()
    }
}

/// Builder for [`PrepareSecRequestContextUpdater`].
///
/// Use this builder to fluently construct an updater for the context.
pub struct PrepareSecRequestContextUpdaterBuilder {
    cik: Option<Cik>,
    max_retries: Option<u32>,
}
impl PrepareSecRequestContextUpdaterBuilder {
    /// Creates a new [`PrepareSecRequestContextUpdaterBuilder`] with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cik: None,
            max_retries: None,
        }
    }

    /// Sets the `max_retries` value inside the context to the provided update value.
    ///
    /// # Arguments
    /// * `max_retries` - The new value for `max_retries`.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Sets the `cik` value inside the context to the provided update value.
    ///
    /// # Arguments
    /// * `cik` - The new value for `cik`.
    #[must_use]
    pub fn cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    /// Builds the [`PrepareSecRequestContextUpdater`] with the specified fields.
    #[must_use]
    pub fn build(self) -> PrepareSecRequestContextUpdater {
        PrepareSecRequestContextUpdater {
            cik: self.cik,
            max_retries: self.max_retries,
        }
    }
}

impl Default for PrepareSecRequestContextUpdaterBuilder {
    /// Returns a new [`PrepareSecRequestContextUpdaterBuilder`] with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    use super::{
        PrepareSecRequestContext, PrepareSecRequestContextUpdater,
        PrepareSecRequestContextUpdaterBuilder,
    };

    #[test]
    fn should_return_reference_to_default_request_context_when_initialized_with_default() {
        let request_context = PrepareSecRequestContext::default();

        let expected_result = &PrepareSecRequestContext::default();

        let result = request_context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_default_as_constructor() {
        let request_context = &PrepareSecRequestContext::default();
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid.");

        let expected_result = &PrepareSecRequestContext {
            cik,
            max_retries: 5,
        };

        let result = request_context.get_context();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_update_context_max_retries_to_specified_value_when_update_contains_specified_value() {
        let mut context = PrepareSecRequestContext::default();
        let update = PrepareSecRequestContextUpdater::builder()
            .max_retries(5)
            .build();
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid.");

        let expected_result = &PrepareSecRequestContext {
            cik,
            max_retries: 5,
        };

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_to_latest_specified_value_when_multiple_updates_in_builder() {
        let mut context = PrepareSecRequestContext::default();
        let update = PrepareSecRequestContextUpdater::builder()
            .max_retries(5)
            .max_retries(10)
            .build();
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid.");

        let expected_result = &PrepareSecRequestContext {
            cik,
            max_retries: 10,
        };

        context.update_context(update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_max_retries_the_default_when_update_contains_a_different_value() {
        let mut context = PrepareSecRequestContext::default();
        let update = PrepareSecRequestContextUpdater::builder()
            .max_retries(5)
            .build();

        let expected_result = 0;

        context.update_context(update);
        let result = context.get_context().max_retries;

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = PrepareSecRequestContext::default();
        let empty_update = PrepareSecRequestContextUpdaterBuilder::default().build();

        let expected_result = &PrepareSecRequestContext::default();

        context.update_context(empty_update);
        let result = context.get_context();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_context_data_trait() {
        implements_auto_traits::<PrepareSecRequestContext>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_context_data_trait() {
        implements_send::<PrepareSecRequestContext>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_context_data_trait() {
        implements_sync::<PrepareSecRequestContext>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_context_data_trait() {
        implements_send::<PrepareSecRequestContext>();
        implements_sync::<PrepareSecRequestContext>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_context_data_trait() {
        implements_sized::<PrepareSecRequestContext>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_context_data_trait() {
        implements_hash::<PrepareSecRequestContext>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_context_data_trait() {
        implements_partial_eq::<PrepareSecRequestContext>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_context_data_trait() {
        implements_eq::<PrepareSecRequestContext>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_context_data_trait() {
        implements_partial_ord::<PrepareSecRequestContext>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_context_data_trait() {
        implements_ord::<PrepareSecRequestContext>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_context_data_trait() {
        implements_default::<PrepareSecRequestContext>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_context_data_trait() {
        implements_debug::<PrepareSecRequestContext>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_context_data_trait() {
        implements_clone::<PrepareSecRequestContext>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_context_data_trait() {
        implements_unpin::<PrepareSecRequestContext>();
    }
}
