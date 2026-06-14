//! # Prepare SEC Request Context
//!
//! Provides the [`PrepareSecRequestContext`] carried alongside the
//! [`PrepareSecRequest`](crate::implementations::states::extract::prepare_sec_request::PrepareSecRequest)
//! state, together with its updater and builder.
//!
//! The context holds the validated [`Cik`] and the retry budget that persist across the
//! state's lifetime, so they survive updates and transitions.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::extract::prepare_sec_request::context::*;
//! use sec::shared::cik::Cik;
//! use state_maschine::prelude::*;
//!
//! let cik = Cik::new("1234567890").expect("A hardcoded valid CIK should always parse");
//! let mut context = PrepareSecRequestContext::new(cik);
//! let update = PrepareSecRequestContextUpdater::builder()
//!     .max_retries(5)
//!     .build();
//!
//! context.update_context(update);
//! assert_eq!(context.max_retries, 5);
//! ```
//!
//! ## See Also
//!
//! - [`crate::traits::state_machine::state::Context`]: Trait that defines context updates and the retry budget.
//! - [`crate::implementations::states::extract::prepare_sec_request`]: Parent module for the preparation state and its data types.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;

use crate::shared::cik::Cik;
use crate::traits::state_machine::state::Context;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
/// Ambient context for the [`PrepareSecRequest`](super::PrepareSecRequest) state.
///
/// Holds the validated [`Cik`] the request targets and the retry budget, both of which
/// persist across updates and transitions.
pub struct PrepareSecRequestContext {
    /// The validated CIK the prepared request targets.
    pub cik: Cik,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl PrepareSecRequestContext {
    /// Creates a new context from a validated CIK, with the retry budget at zero.
    #[must_use]
    pub const fn new(cik: Cik) -> Self {
        Self {
            cik,
            max_retries: 0,
        }
    }
}

impl Context for PrepareSecRequestContext {
    /// Returns the maximum number of retries allowed for the SEC request.
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContext for PrepareSecRequestContext {
    type UpdateType = PrepareSecRequestContextUpdater;

    /// Returns a reference to the current context.
    fn context(&self) -> &Self {
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
/// Partial update for a [`PrepareSecRequestContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
pub struct PrepareSecRequestContextUpdater {
    /// Optional new value for the validated CIK.
    pub cik: Option<Cik>,
    /// Optional new value for the retry budget.
    pub max_retries: Option<u32>,
}

impl PrepareSecRequestContextUpdater {
    /// Creates a new builder for constructing [`PrepareSecRequestContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> PrepareSecRequestContextUpdaterBuilder {
        PrepareSecRequestContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`PrepareSecRequestContextUpdater`].
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

    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::*;

    use super::{
        PrepareSecRequestContext, PrepareSecRequestContextUpdater,
        PrepareSecRequestContextUpdaterBuilder,
    };
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;

    fn create_test_context() -> PrepareSecRequestContext {
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid");
        PrepareSecRequestContext::new(cik)
    }

    #[test]
    fn should_update_context_max_retries_to_specified_value_when_update_contains_specified_value() {
        let mut context = create_test_context();
        let update = PrepareSecRequestContextUpdater::builder()
            .max_retries(5)
            .build();
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid");

        let expected_result = &PrepareSecRequestContext {
            cik,
            max_retries: 5,
        };

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_to_latest_specified_value_when_multiple_updates_in_builder() {
        let mut context = create_test_context();
        let update = PrepareSecRequestContextUpdater::builder()
            .max_retries(5)
            .max_retries(10)
            .build();
        let cik =
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect("Hardcoded CIK should always be valid");

        let expected_result = &PrepareSecRequestContext {
            cik,
            max_retries: 10,
        };

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_leave_context_max_retries_the_default_when_update_contains_a_different_value() {
        let mut context = create_test_context();
        let update = PrepareSecRequestContextUpdater::builder()
            .max_retries(5)
            .build();

        let expected_result = 0;

        context.update_context(update);
        let result = context.context().max_retries;

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = create_test_context();
        let empty_update = PrepareSecRequestContextUpdaterBuilder::default().build();

        let expected_result = &create_test_context();

        context.update_context(empty_update);
        let result = context.context();

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
