//! # `CreateFinancialStatementsOutput` Module
//!
//! This module defines the output data structure and updater patterns for the `CreateFinancialStatements` state
//! within the SEC transform state machine.
//!
//! ## Types
//! - [`CreateFinancialStatementsOutput`]: Placeholder unit struct for the financial statements output.
//! - [`CreateFinancialStatementsOutputUpdater`]: Updater type for modifying the output data in a controlled manner.
//! - [`CreateFinancialStatementsOutputUpdaterBuilder`]: Builder for constructing updater instances.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`CreateFinancialStatements`](crate::implementations::states::transform::create_financial_statements) to produce output data.
//!
//! ## Usage
//! This module is a placeholder for the output phase of financial statement creation.
//! The output struct will be extended with financial statement fields in a future iteration.
//!
//! ## See Also
//! - [`input`](super::input): Input data structure for company data.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.
//!
//! ## Examples
//! See the unit tests in this module for usage patterns.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::traits::state_machine::state::StateData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
/// Placeholder output data for the Create Financial Statements state.
///
/// This unit struct serves as a scaffold for the financial statement creation output.
/// It will be extended with concrete financial statement fields in a future iteration.
pub struct CreateFinancialStatementsOutput;

impl StateData for CreateFinancialStatementsOutput {
    /// Updates the state data using the provided updater.
    ///
    /// Currently a no-op since this is a placeholder output with no fields.
    fn update_state(&mut self, _updates: Self::UpdateType) -> Result<(), StateError> {
        Ok(())
    }
}

impl SMStateData for CreateFinancialStatementsOutput {
    type UpdateType = CreateFinancialStatementsOutputUpdater;

    /// Returns a reference to the current state data, which represents the output data of this state.
    fn state(&self) -> &Self {
        self
    }

    /// Delegates to the SEC [`StateData::update_state`] implementation.
    ///
    /// # Panics
    /// Panics if the fallible SEC update returns an error.
    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Err(e) = <Self as StateData>::update_state(self, updates) {
            panic!("StateData::update_state failed: {e}")
        }
    }
}

impl fmt::Display for CreateFinancialStatementsOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\t(placeholder output)")
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
/// Updater for [`CreateFinancialStatementsOutput`].
///
/// This struct is a placeholder updater with no fields. It will be extended
/// alongside the output struct in a future iteration.
pub struct CreateFinancialStatementsOutputUpdater;

impl CreateFinancialStatementsOutputUpdater {
    /// Creates a new builder for constructing [`CreateFinancialStatementsOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> CreateFinancialStatementsOutputUpdaterBuilder {
        CreateFinancialStatementsOutputUpdaterBuilder::new()
    }
}

/// Builder for [`CreateFinancialStatementsOutputUpdater`].
///
/// This builder is a placeholder. It will be extended with fields
/// alongside the output struct in a future iteration.
pub struct CreateFinancialStatementsOutputUpdaterBuilder;

impl CreateFinancialStatementsOutputUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Builds the updater instance from the builder.
    #[must_use]
    pub const fn build(self) -> CreateFinancialStatementsOutputUpdater {
        CreateFinancialStatementsOutputUpdater
    }
}

impl Default for CreateFinancialStatementsOutputUpdaterBuilder {
    /// Returns a new updater builder with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;
    use state_maschine::prelude::StateData as SMStateData;

    use super::CreateFinancialStatementsOutput;

    #[test]
    fn should_return_reference_to_default_output_data_when_initialized_with_default() {
        let output = CreateFinancialStatementsOutput::default();

        let expected_result = &CreateFinancialStatementsOutput;

        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_output_data_trait() {
        implements_auto_traits::<CreateFinancialStatementsOutput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_output_data_trait() {
        implements_send::<CreateFinancialStatementsOutput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_output_data_trait() {
        implements_sync::<CreateFinancialStatementsOutput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_output_data_trait() {
        implements_send::<CreateFinancialStatementsOutput>();
        implements_sync::<CreateFinancialStatementsOutput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_output_data_trait() {
        implements_sized::<CreateFinancialStatementsOutput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_output_data_trait() {
        implements_hash::<CreateFinancialStatementsOutput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_output_data_trait() {
        implements_partial_eq::<CreateFinancialStatementsOutput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_output_data_trait() {
        implements_eq::<CreateFinancialStatementsOutput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_output_data_trait() {
        implements_partial_ord::<CreateFinancialStatementsOutput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_output_data_trait() {
        implements_ord::<CreateFinancialStatementsOutput>();
    }

    const fn implements_default<T: Default>() {}
    #[test]
    const fn should_implement_default_when_implementing_output_data_trait() {
        implements_default::<CreateFinancialStatementsOutput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_output_data_trait() {
        implements_debug::<CreateFinancialStatementsOutput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_output_data_trait() {
        implements_clone::<CreateFinancialStatementsOutput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_output_data_trait() {
        implements_unpin::<CreateFinancialStatementsOutput>();
    }
}
