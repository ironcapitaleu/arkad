//! # Create Financial Statements Input
//!
//! Provides the [`CreateFinancialStatementsInput`] fed into the
//! [`CreateFinancialStatements`](crate::implementations::states::transform::create_financial_statements::CreateFinancialStatements)
//! state, along with its updater and builder.
//!
//! It carries the [`CompanyData`] produced by the parsing state, from which the statements are
//! built. The produced output lives in [`output`](super::output).
//!
//! ## See Also
//!
//! - [`output`](super::output): The financial statements produced from this input.
//! - [`crate::shared::financial::company_data`]: The [`CompanyData`] type carried here.

use std::fmt;
use std::hash::{Hash, Hasher};

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::financial::company_data::CompanyData;
use crate::traits::state_machine::state::StateData;

// Deviation: `CompanyData` uses manual `Hash`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`
// implementations because it contains a `HashMap`. This struct delegates to those
// manual implementations rather than deriving them.
#[derive(Debug, Clone, Serialize)]
/// Input data for the [`CreateFinancialStatements`](super::super::CreateFinancialStatements) state.
///
/// Holds the [`CompanyData`] produced by the parsing state, from which financial statements
/// are built.
pub struct CreateFinancialStatementsInput {
    /// The company data containing parsed financial facts.
    pub company_data: CompanyData,
}

impl CreateFinancialStatementsInput {
    /// Creates input data from parsed company data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use sec::shared::cik::Cik;
    /// use sec::shared::financial::company_data::CompanyData;
    /// use sec::shared::financial::entity_name::EntityName;
    /// use sec::implementations::states::transform::create_financial_statements::data::input::CreateFinancialStatementsInput;
    ///
    /// let company_data = CompanyData::new(
    ///     Cik::new("0000320193").expect("Hardcoded CIK should always be valid"),
    ///     EntityName::new("Apple Inc."),
    ///     HashMap::new(),
    /// );
    /// let input = CreateFinancialStatementsInput::new(company_data);
    /// ```
    #[must_use]
    pub const fn new(company_data: CompanyData) -> Self {
        Self { company_data }
    }

    /// Returns a reference to the company data.
    #[must_use]
    pub const fn company_data(&self) -> &CompanyData {
        &self.company_data
    }
}

impl StateData for CreateFinancialStatementsInput {
    /// Updates the state data using the provided updater.
    ///
    /// If `company_data` is `Some`, updates the company data; otherwise, leaves it unchanged.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(company_data) = updates.company_data {
            self.company_data = company_data;
        }
        Ok(())
    }
}

impl SMStateData for CreateFinancialStatementsInput {
    type UpdateType = CreateFinancialStatementsInputUpdater;

    /// Returns a reference to the current state data, which represents the input data of this state.
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

impl PartialEq for CreateFinancialStatementsInput {
    fn eq(&self, other: &Self) -> bool {
        self.company_data == other.company_data
    }
}

impl Eq for CreateFinancialStatementsInput {}

// Deviation: delegates to `CompanyData`'s manual `Hash` implementation.
impl Hash for CreateFinancialStatementsInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.company_data.hash(state);
    }
}

impl PartialOrd for CreateFinancialStatementsInput {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Delegates to `CompanyData`'s manual `Ord` implementation (orders by `cik`, then `entity_name`).
impl Ord for CreateFinancialStatementsInput {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.company_data.cmp(&other.company_data)
    }
}

impl fmt::Display for CreateFinancialStatementsInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tCompany Data: {}", self.company_data)
    }
}

#[derive(Debug, Clone)]
/// Partial update for a [`CreateFinancialStatementsInput`].
///
/// When `company_data` is `None` the input is left unchanged.
pub struct CreateFinancialStatementsInputUpdater {
    /// Optional new value for the company data.
    pub company_data: Option<CompanyData>,
}

impl CreateFinancialStatementsInputUpdater {
    /// Creates a new builder for constructing [`CreateFinancialStatementsInputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> CreateFinancialStatementsInputUpdaterBuilder {
        CreateFinancialStatementsInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`CreateFinancialStatementsInputUpdater`].
pub struct CreateFinancialStatementsInputUpdaterBuilder {
    company_data: Option<CompanyData>,
}

impl CreateFinancialStatementsInputUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self { company_data: None }
    }

    /// Sets the company data value to the one to be updated to.
    ///
    /// # Arguments
    ///
    /// * `company_data` - The new [`CompanyData`] value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn company_data(mut self, company_data: CompanyData) -> Self {
        self.company_data = Some(company_data);
        self
    }

    /// Builds the updater instance from the builder.
    #[must_use]
    pub fn build(self) -> CreateFinancialStatementsInputUpdater {
        CreateFinancialStatementsInputUpdater {
            company_data: self.company_data,
        }
    }
}

impl Default for CreateFinancialStatementsInputUpdaterBuilder {
    /// Returns a new updater builder with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::hash::Hash;

    use pretty_assertions::{assert_eq, assert_ne};
    use state_maschine::prelude::StateData as SMStateData;

    use super::{CreateFinancialStatementsInput, CreateFinancialStatementsInputUpdaterBuilder};
    use crate::shared::cik::Cik;
    use crate::shared::cik::constants::BERKSHIRE_HATHAWAY_CIK_RAW;
    use crate::shared::financial::company_data::CompanyData;
    use crate::shared::financial::entity_name::EntityName;
    use crate::traits::state_machine::state::StateData;

    fn test_input() -> CreateFinancialStatementsInput {
        let cik = Cik::new(BERKSHIRE_HATHAWAY_CIK_RAW).expect(
            "Given a valid hardcoded CIK, the creation of a CIK object should always succeed",
        );
        CreateFinancialStatementsInput::new(CompanyData::new(
            cik,
            EntityName::new("BERKSHIRE HATHAWAY INC"),
            HashMap::new(),
        ))
    }

    fn create_custom_company_data() -> CompanyData {
        let cik = Cik::new("0000320193").expect(
            "Given a valid hardcoded CIK, the creation of a CIK object should always succeed",
        );
        CompanyData::new(cik, EntityName::new("Apple Inc."), HashMap::new())
    }

    #[test]
    fn should_return_reference_to_default_input_data_when_initialized_with_default() {
        let default_input = test_input();

        let expected_result = &test_input();

        let result = default_input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_input_with_custom_data_when_using_new_as_constructor() {
        let custom_input = &CreateFinancialStatementsInput::new(create_custom_company_data());

        let default_input = &test_input();

        let result = custom_input.state();

        assert_ne!(result, default_input);
    }

    #[test]
    fn should_update_state_data_when_update_contains_new_company_data() {
        let mut state_data = test_input();
        let new_company_data = create_custom_company_data();
        let update = CreateFinancialStatementsInputUpdaterBuilder::default()
            .company_data(new_company_data.clone())
            .build();

        let expected_result = &CreateFinancialStatementsInput::new(new_company_data);

        StateData::update_state(&mut state_data, update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_state_data_unchanged_when_empty_update() {
        let mut state_data = test_input();
        let empty_update = CreateFinancialStatementsInputUpdaterBuilder::default().build();

        let expected_result = &test_input();

        StateData::update_state(&mut state_data, empty_update)
            .expect("Update with valid 'update' value should always succeed");
        let result = state_data.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_input_data_trait() {
        implements_auto_traits::<CreateFinancialStatementsInput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_input_data_trait() {
        implements_send::<CreateFinancialStatementsInput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_input_data_trait() {
        implements_sync::<CreateFinancialStatementsInput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_input_data_trait() {
        implements_send::<CreateFinancialStatementsInput>();
        implements_sync::<CreateFinancialStatementsInput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_input_data_trait() {
        implements_sized::<CreateFinancialStatementsInput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_input_data_trait() {
        implements_hash::<CreateFinancialStatementsInput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_input_data_trait() {
        implements_partial_eq::<CreateFinancialStatementsInput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_input_data_trait() {
        implements_eq::<CreateFinancialStatementsInput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_input_data_trait() {
        implements_partial_ord::<CreateFinancialStatementsInput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_input_data_trait() {
        implements_ord::<CreateFinancialStatementsInput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_input_data_trait() {
        implements_debug::<CreateFinancialStatementsInput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_input_data_trait() {
        implements_clone::<CreateFinancialStatementsInput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_input_data_trait() {
        implements_unpin::<CreateFinancialStatementsInput>();
    }
}
