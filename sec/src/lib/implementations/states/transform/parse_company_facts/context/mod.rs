//! # Parse Company Facts Context
//!
//! Provides the [`ParseCompanyFactsContext`] carried alongside the
//! [`ParseCompanyFacts`](crate::implementations::states::transform::parse_company_facts::ParseCompanyFacts)
//! state, together with its updater and builder.
//!
//! The context tracks the [`Cik`] being processed and the retry budget, both of which persist
//! across updates and transitions.
//!
//! ## Usage
//!
//! ```rust
//! use sec::implementations::states::transform::parse_company_facts::context::*;
//! use sec::prelude::*;
//! use sec::shared::cik::Cik;
//!
//! let cik = Cik::new("0001067983").expect("A hardcoded valid CIK should always parse");
//! let mut context = ParseCompanyFactsContext::new(cik);
//! let update = ParseCompanyFactsContextUpdater::builder()
//!     .max_retries(3)
//!     .build();
//!
//! context.update_context(update);
//! assert_eq!(context.max_retries(), 3);
//! ```
//!
//! ## See Also
//!
//! - [`crate::traits::state_machine::state::Context`]: Trait that defines context updates and the retry budget.
//! - [`crate::implementations::states::transform::parse_company_facts`]: Parent module for the parsing state and its data types.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::Context as SMContext;

use crate::shared::cik::Cik;
use crate::traits::state_machine::state::Context;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
/// Ambient context for the [`ParseCompanyFacts`](super::ParseCompanyFacts) state.
///
/// Tracks the [`Cik`] of the company being parsed and the retry budget, both of which persist
/// across updates and transitions.
pub struct ParseCompanyFactsContext {
    /// The validated CIK for the company whose facts are being parsed.
    pub cik: Cik,
    /// Maximum number of times the state may be retried on failure.
    pub max_retries: u32,
}

impl ParseCompanyFactsContext {
    /// Creates a new context from a CIK, with the retry budget at zero.
    #[must_use]
    pub const fn new(cik: Cik) -> Self {
        Self {
            cik,
            max_retries: 0,
        }
    }

    /// Returns a reference to the current CIK in the context.
    #[must_use]
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }
}

impl Context for ParseCompanyFactsContext {
    /// Returns the maximum number of retries allowed for company facts parsing.
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContext for ParseCompanyFactsContext {
    type UpdateType = ParseCompanyFactsContextUpdater;

    /// Returns a reference to the current context.
    fn context(&self) -> &Self {
        self
    }

    /// Updates the context fields using the provided updater.
    ///
    /// Only fields set in the updater will be changed.
    fn update_context(&mut self, updates: Self::UpdateType) {
        if let Some(cik) = updates.cik {
            self.cik = cik;
        }
        if let Some(max_retries) = updates.max_retries {
            self.max_retries = max_retries;
        }
    }
}

impl fmt::Display for ParseCompanyFactsContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CIK: {}, Max Retries: {}", self.cik, self.max_retries)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Partial update for a [`ParseCompanyFactsContext`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
pub struct ParseCompanyFactsContextUpdater {
    /// Optional new value for the CIK.
    pub cik: Option<Cik>,
    /// Optional new value for the retry budget.
    pub max_retries: Option<u32>,
}

impl ParseCompanyFactsContextUpdater {
    /// Creates a new builder for constructing [`ParseCompanyFactsContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ParseCompanyFactsContextUpdaterBuilder {
        ParseCompanyFactsContextUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`ParseCompanyFactsContextUpdater`].
pub struct ParseCompanyFactsContextUpdaterBuilder {
    cik: Option<Cik>,
    max_retries: Option<u32>,
}

impl ParseCompanyFactsContextUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cik: None,
            max_retries: None,
        }
    }

    /// Sets the CIK value inside the context to the provided update value.
    ///
    /// # Arguments
    /// * `cik` - The new validated [`Cik`] value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    /// Sets the max retries value inside the context to the provided update value.
    ///
    /// # Arguments
    /// * `max_retries` - The new maximum number of retries.
    #[must_use]
    pub const fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Builds the updater with the specified fields.
    #[must_use]
    pub fn build(self) -> ParseCompanyFactsContextUpdater {
        ParseCompanyFactsContextUpdater {
            cik: self.cik,
            max_retries: self.max_retries,
        }
    }
}

impl Default for ParseCompanyFactsContextUpdaterBuilder {
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
        ParseCompanyFactsContext, ParseCompanyFactsContextUpdater,
        ParseCompanyFactsContextUpdaterBuilder,
    };
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;

    fn test_context() -> ParseCompanyFactsContext {
        ParseCompanyFactsContext::new(
            Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW)
                .expect("Hardcoded Berkshire Hathaway CIK should always be valid"),
        )
    }

    #[test]
    fn should_return_reference_to_default_context_when_initialized_with_default() {
        let context = test_context();

        let expected_result = &test_context();

        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let cik = Cik::new("0000000001").expect("Hardcoded CIK should always be valid");
        let context = &ParseCompanyFactsContext::new(cik);

        let expected_result = &test_context();

        let result = context.context();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_update_context_max_retries_when_update_contains_specified_value() {
        let mut context = test_context();
        let update = ParseCompanyFactsContextUpdater::builder()
            .max_retries(5)
            .build();

        let expected_result = 5;

        context.update_context(update);
        let result = context.max_retries;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_context_cik_when_update_contains_specified_cik() {
        let new_cik = Cik::new("0000000001").expect("Hardcoded CIK should always be valid");
        let mut context = test_context();
        let update = ParseCompanyFactsContextUpdater::builder()
            .cik(new_cik.clone())
            .build();

        let expected_result = &new_cik;

        context.update_context(update);
        let result = context.cik();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = test_context();
        let empty_update = ParseCompanyFactsContextUpdaterBuilder::default().build();

        let expected_result = &test_context();

        context.update_context(empty_update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_updater_contains_max_retries() {
        let mut context = test_context();
        let update = ParseCompanyFactsContextUpdater::builder()
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
        implements_auto_traits::<ParseCompanyFactsContext>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_context_data_trait() {
        implements_send::<ParseCompanyFactsContext>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_context_data_trait() {
        implements_sync::<ParseCompanyFactsContext>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_context_data_trait() {
        implements_send::<ParseCompanyFactsContext>();
        implements_sync::<ParseCompanyFactsContext>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_context_data_trait() {
        implements_sized::<ParseCompanyFactsContext>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_context_data_trait() {
        implements_hash::<ParseCompanyFactsContext>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_context_data_trait() {
        implements_partial_eq::<ParseCompanyFactsContext>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_context_data_trait() {
        implements_eq::<ParseCompanyFactsContext>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_context_data_trait() {
        implements_partial_ord::<ParseCompanyFactsContext>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_context_data_trait() {
        implements_ord::<ParseCompanyFactsContext>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_context_data_trait() {
        implements_debug::<ParseCompanyFactsContext>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_context_data_trait() {
        implements_clone::<ParseCompanyFactsContext>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_context_data_trait() {
        implements_unpin::<ParseCompanyFactsContext>();
    }
}
