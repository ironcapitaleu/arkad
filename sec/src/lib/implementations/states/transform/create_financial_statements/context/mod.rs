//! # Create Financial Statements Context Module
//!
//! This module defines the context structures and updaters for the [`CreateFinancialStatements`](../mod.rs) state in the SEC transform workflow.
//!
//! The context provides stateful information required during financial statement creation, such as the CIK and retry configuration.
//! It is designed to be used with the [`Context`] trait, enabling ergonomic context management and updates within state machines.
//!
//! ## Components
//! - [`CreateFinancialStatementsContext`]: Holds the current context for financial statement creation, including the CIK and retry count.
//! - [`CreateFinancialStatementsContextUpdater`]: Updater type for modifying context fields in a controlled way.
//! - [`CreateFinancialStatementsContextUpdaterBuilder`]: Builder for constructing context updaters with a fluent API.
//!
//! ## Usage
//! The context is used by the [`CreateFinancialStatements`](../mod.rs) state to track the company being processed and manage retry logic.
//! It supports updates via the builder pattern, making it easy to compose context changes in state machine workflows.
//!
//! ## Example
//! ```rust
//! use sec::implementations::states::transform::create_financial_statements::context::*;
//! use state_maschine::prelude::*;
//!
//! let mut context = CreateFinancialStatementsContext::default();
//! let update = CreateFinancialStatementsContextUpdater::builder()
//!     .cik(sec::shared::cik::Cik::new("0000000001").unwrap())
//!     .build();
//! context.update_context(update);
//! assert_eq!(context.cik().value(), "0000000001");
//! ```
//!
//! ## See Also
//! - [`crate::traits::state_machine::state::Context`]: Trait for context management in states.
//! - [`crate::implementations::states::transform::create_financial_statements`]: Parent module for financial statement creation state and data types.

use std::fmt;

use state_maschine::prelude::Context as SMContext;

use crate::shared::cik::Cik;
use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
use crate::traits::state_machine::state::Context;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, serde::Serialize)]
/// State context for the Create Financial Statements state.
///
/// The default instance uses the CIK for Berkshire Hathaway (CIK: 1067983).
pub struct CreateFinancialStatementsContext {
    /// The CIK identifying the company whose financial statements are being created.
    pub cik: Cik,
    /// The maximum number of retries allowed for this state.
    pub max_retries: u32,
}

impl CreateFinancialStatementsContext {
    /// Creates a new instance of the state context for the Create Financial Statements state.
    ///
    /// # Arguments
    /// * `cik` - A validated [`Cik`] identifying the company.
    ///
    /// # Returns
    /// A new `CreateFinancialStatementsContext` with the provided CIK and default retry count.
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

impl Context for CreateFinancialStatementsContext {
    /// Returns the maximum number of retries allowed for financial statement creation.
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SMContext for CreateFinancialStatementsContext {
    type UpdateType = CreateFinancialStatementsContextUpdater;

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

impl Default for CreateFinancialStatementsContext {
    /// Returns a default context using the CIK for Berkshire Hathaway (CIK: 1067983).
    fn default() -> Self {
        Self::new(Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect(
            "Given a valid hardcoded CIK, the creation of a CIK object should always succeed",
        ))
    }
}

impl fmt::Display for CreateFinancialStatementsContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CIK: {}", self.cik)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for the state context.
///
/// Using this struct allows to update fields of `CreateFinancialStatementsContext` in a controlled way.
pub struct CreateFinancialStatementsContextUpdater {
    /// Optional new value for the CIK.
    pub cik: Option<Cik>,
    /// Optional new maximum retries value.
    pub max_retries: Option<u32>,
}

impl CreateFinancialStatementsContextUpdater {
    /// Creates a new builder for constructing [`CreateFinancialStatementsContextUpdater`] instances.
    #[must_use]
    pub const fn builder() -> CreateFinancialStatementsContextUpdaterBuilder {
        CreateFinancialStatementsContextUpdaterBuilder::new()
    }
}

/// Builder for `CreateFinancialStatementsContextUpdater`.
///
/// Use this builder to fluently construct an updater for the context.
pub struct CreateFinancialStatementsContextUpdaterBuilder {
    cik: Option<Cik>,
    max_retries: Option<u32>,
}

impl CreateFinancialStatementsContextUpdaterBuilder {
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
    /// * `cik` - A validated [`Cik`] representing the new CIK.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
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
    pub fn build(self) -> CreateFinancialStatementsContextUpdater {
        CreateFinancialStatementsContextUpdater {
            cik: self.cik,
            max_retries: self.max_retries,
        }
    }
}

impl Default for CreateFinancialStatementsContextUpdaterBuilder {
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
        CreateFinancialStatementsContext, CreateFinancialStatementsContextUpdater,
        CreateFinancialStatementsContextUpdaterBuilder,
    };
    use crate::shared::cik::Cik;

    #[test]
    fn should_return_reference_to_default_context_when_initialized_with_default() {
        let context = CreateFinancialStatementsContext::default();

        let expected_result = &CreateFinancialStatementsContext::default();

        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_context_with_custom_data_when_using_new_as_constructor() {
        let custom_cik = Cik::new("0000000001").expect(
            "Given a valid hardcoded CIK, the creation of a CIK object should always succeed",
        );
        let context = &CreateFinancialStatementsContext::new(custom_cik);

        let expected_result = &CreateFinancialStatementsContext::default();

        let result = context.context();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_update_context_cik_when_update_contains_new_cik() {
        let mut context = CreateFinancialStatementsContext::default();
        let new_cik = Cik::new("0000000001").expect(
            "Given a valid hardcoded CIK, the creation of a CIK object should always succeed",
        );
        let update = CreateFinancialStatementsContextUpdater::builder()
            .cik(new_cik.clone())
            .build();

        let expected_result = &CreateFinancialStatementsContext::new(new_cik);

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_cik_to_latest_specified_value_when_multiple_updates_in_builder() {
        let mut context = CreateFinancialStatementsContext::default();
        let first_cik = Cik::new("0000000001").expect(
            "Given a valid hardcoded CIK, the creation of a CIK object should always succeed",
        );
        let latest_cik = Cik::new("0000000002").expect(
            "Given a valid hardcoded CIK, the creation of a CIK object should always succeed",
        );
        let update = CreateFinancialStatementsContextUpdater::builder()
            .cik(first_cik)
            .cik(latest_cik.clone())
            .build();

        let expected_result = &CreateFinancialStatementsContext::new(latest_cik);

        context.update_context(update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_context_unchanged_when_empty_update() {
        let mut context = CreateFinancialStatementsContext::default();
        let empty_update = CreateFinancialStatementsContextUpdaterBuilder::default().build();

        let expected_result = &CreateFinancialStatementsContext::default();

        context.update_context(empty_update);
        let result = context.context();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_max_retries_when_updater_contains_max_retries() {
        let mut context = CreateFinancialStatementsContext::default();
        let update = CreateFinancialStatementsContextUpdater::builder()
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
        implements_auto_traits::<CreateFinancialStatementsContext>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_context_data_trait() {
        implements_send::<CreateFinancialStatementsContext>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_context_data_trait() {
        implements_sync::<CreateFinancialStatementsContext>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_context_data_trait() {
        implements_send::<CreateFinancialStatementsContext>();
        implements_sync::<CreateFinancialStatementsContext>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_context_data_trait() {
        implements_sized::<CreateFinancialStatementsContext>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_context_data_trait() {
        implements_hash::<CreateFinancialStatementsContext>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_context_data_trait() {
        implements_partial_eq::<CreateFinancialStatementsContext>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_context_data_trait() {
        implements_eq::<CreateFinancialStatementsContext>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_context_data_trait() {
        implements_partial_ord::<CreateFinancialStatementsContext>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_context_data_trait() {
        implements_ord::<CreateFinancialStatementsContext>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_context_data_trait() {
        implements_default::<CreateFinancialStatementsContext>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_context_data_trait() {
        implements_debug::<CreateFinancialStatementsContext>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_context_data_trait() {
        implements_clone::<CreateFinancialStatementsContext>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_context_data_trait() {
        implements_unpin::<CreateFinancialStatementsContext>();
    }
}
