//! # `ParseCompanyFactsOutput` Module
//!
//! This module defines the output data structure and updater patterns for the `ParseCompanyFacts` state
//! within the SEC transformation state machine. It encapsulates the parsed company data produced
//! by resolving XBRL concepts from SEC Company Facts JSON.
//!
//! ## Types
//! - [`ParseCompanyFactsOutput`]: Holds the parsed [`CompanyData`] after successful fact resolution.
//! - [`ParseCompanyFactsOutputUpdater`]: Updater type for modifying the output data in a controlled manner.
//! - [`ParseCompanyFactsOutputUpdaterBuilder`]: Builder for constructing updater instances with optional fields.
//!
//! ## Integration
//! - Implements [`StateData`](state_maschine::state_machine::state::StateData) for compatibility with the state machine framework.
//! - Used by [`ParseCompanyFacts`](crate::implementations::states::transform::parse_company_facts) to produce and update parsed output.
//!
//! ## Usage
//! This module is intended for use in the output phase of company facts parsing. It supports
//! builder-based updates and integrates with the state machine's updater and state data traits.
//!
//! ## See Also
//! - [`input`](super::input): Input data structure for raw JSON responses.
//! - [`crate::shared::financial::company_data::CompanyData`]: The resolved company data type.
//! - [`state_maschine::prelude::StateData`]: Trait for state data integration.

use std::fmt;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::financial::company_data::CompanyData;
use crate::traits::state_machine::state::StateData;

/// Output data containing parsed company financial data.
///
/// This struct holds a [`CompanyData`] value, produced by the `ParseCompanyFacts` state
/// after successfully resolving XBRL concepts from SEC Company Facts JSON. It is used as
/// output in the SEC transformation state machine, and supports builder-based updates and
/// integration with the state machine framework.
#[derive(Debug, Clone)]
pub struct ParseCompanyFactsOutput {
    /// The parsed company financial data.
    pub company_data: CompanyData,
}

impl ParseCompanyFactsOutput {
    /// Creates a new instance of the output data for the company facts parsing state.
    #[must_use]
    pub const fn new(company_data: CompanyData) -> Self {
        Self { company_data }
    }

    /// Returns a reference to the parsed company data.
    #[must_use]
    pub const fn company_data(&self) -> &CompanyData {
        &self.company_data
    }
}

impl PartialEq for ParseCompanyFactsOutput {
    fn eq(&self, other: &Self) -> bool {
        self.company_data == other.company_data
    }
}

impl Eq for ParseCompanyFactsOutput {}

// Deviation: `CompanyData` has a manual `Hash` implementation that only hashes
// `cik` and `entity_name`, so we delegate to that.
impl std::hash::Hash for ParseCompanyFactsOutput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.company_data.hash(state);
    }
}

impl PartialOrd for ParseCompanyFactsOutput {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Deviation: `CompanyData` has a manual `Ord` implementation based on `cik` only.
impl Ord for ParseCompanyFactsOutput {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.company_data.cmp(&other.company_data)
    }
}

impl serde::Serialize for ParseCompanyFactsOutput {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ParseCompanyFactsOutput", 1)?;
        state.serialize_field("company_data", &self.company_data)?;
        state.end()
    }
}

impl StateData for ParseCompanyFactsOutput {
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

impl SMStateData for ParseCompanyFactsOutput {
    type UpdateType = ParseCompanyFactsOutputUpdater;

    /// Returns a reference to the current state data, which represents the output data of this state.
    fn state(&self) -> &Self {
        self
    }

    /// Panics unconditionally — SEC state data uses fallible updates.
    fn update_state(&mut self, _updates: Self::UpdateType) {
        unimplemented!(
            "SEC state data uses fallible updates. \
             Call sec::StateData::update_state instead"
        )
    }
}

impl fmt::Display for ParseCompanyFactsOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\tCompany Data: {}", self.company_data)
    }
}

#[derive(Debug, Clone)]
/// Updater for [`ParseCompanyFactsOutput`].
///
/// This struct is used to specify updates to the output data in a controlled, partial manner.
/// Fields set to `None` will not be updated.
pub struct ParseCompanyFactsOutputUpdater {
    /// Optional new value for the company data.
    pub company_data: Option<CompanyData>,
}

impl ParseCompanyFactsOutputUpdater {
    /// Creates a new builder for constructing [`ParseCompanyFactsOutputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> ParseCompanyFactsOutputUpdaterBuilder {
        ParseCompanyFactsOutputUpdaterBuilder::new()
    }
}

/// Builder for [`ParseCompanyFactsOutputUpdater`].
///
/// This builder allows for ergonomic and explicit construction of updater instances,
/// supporting method chaining and optional fields. Use `.build()` to produce the updater.
pub struct ParseCompanyFactsOutputUpdaterBuilder {
    company_data: Option<CompanyData>,
}

impl ParseCompanyFactsOutputUpdaterBuilder {
    /// Creates a new updater builder with no fields set.
    #[must_use]
    pub const fn new() -> Self {
        Self { company_data: None }
    }

    /// Sets the company data for the updater.
    ///
    /// # Arguments
    ///
    /// * `company_data` - The new [`CompanyData`] to set in the output data.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn company_data(mut self, company_data: CompanyData) -> Self {
        self.company_data = Some(company_data);
        self
    }

    /// Builds the updater instance from the builder.
    #[must_use]
    pub fn build(self) -> ParseCompanyFactsOutputUpdater {
        ParseCompanyFactsOutputUpdater {
            company_data: self.company_data,
        }
    }
}

impl Default for ParseCompanyFactsOutputUpdaterBuilder {
    /// Returns a new updater builder with no fields set.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::{assert_eq, assert_ne};

    use super::{ParseCompanyFactsOutput, ParseCompanyFactsOutputUpdaterBuilder};
    use crate::shared::cik::Cik;
    use crate::shared::financial::company_data::CompanyData;
    use crate::shared::financial::entity_name::EntityName;
    use crate::traits::state_machine::state::StateData;
    use state_maschine::prelude::StateData as SMStateData;

    fn create_test_output(entity_name: &str) -> ParseCompanyFactsOutput {
        let cik = Cik::new("0000320193").expect("Hardcoded CIK should always be valid");
        let company_data = CompanyData::new(cik, EntityName::new(entity_name), HashMap::new());
        ParseCompanyFactsOutput::new(company_data)
    }

    #[test]
    fn should_return_reference_to_output_data_when_initialized() {
        let output_data = create_test_output("Apple Inc.");

        let expected_result = &create_test_output("Apple Inc.");

        let result = output_data.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_different_output_data_with_custom_data_when_using_new() {
        let output_data = create_test_output("Microsoft Corp.");

        let expected_result = &create_test_output("Apple Inc.");

        let result = output_data.state();

        assert_ne!(result, expected_result);
    }

    #[test]
    fn should_return_company_data_reference_when_accessed() {
        let output_data = create_test_output("Apple Inc.");

        let expected_result = "Apple Inc.";

        let result = output_data.company_data().entity_name().value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_output_data_when_updater_contains_company_data() {
        let mut output_data = create_test_output("Apple Inc.");
        let new_company_data = CompanyData::new(
            Cik::new("0000789019").expect("Hardcoded CIK should always be valid"),
            EntityName::new("Microsoft Corp."),
            HashMap::new(),
        );
        let update = ParseCompanyFactsOutputUpdaterBuilder::default()
            .company_data(new_company_data)
            .build();

        let expected_result = "Microsoft Corp.";

        StateData::update_state(&mut output_data, update)
            .expect("Update with valid company data should succeed");
        let result = output_data.company_data().entity_name().value();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_data_unchanged_when_empty_update() {
        let mut output_data = create_test_output("Apple Inc.");
        let empty_update = ParseCompanyFactsOutputUpdaterBuilder::default().build();

        let expected_result = &create_test_output("Apple Inc.");

        StateData::update_state(&mut output_data, empty_update)
            .expect("Update with empty updater should succeed");
        let result = output_data.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_still_implement_auto_traits_when_implementing_output_data_trait() {
        implements_auto_traits::<ParseCompanyFactsOutput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_implement_send_when_implementing_output_data_trait() {
        implements_send::<ParseCompanyFactsOutput>();
    }

    #[test]
    const fn should_implement_sync_when_implementing_output_data_trait() {
        implements_sync::<ParseCompanyFactsOutput>();
    }

    #[test]
    const fn should_be_thread_safe_when_implementing_output_data_trait() {
        implements_send::<ParseCompanyFactsOutput>();
        implements_sync::<ParseCompanyFactsOutput>();
    }

    const fn implements_sized<T: Sized>() {}
    #[test]
    const fn should_be_sized_when_implementing_output_data_trait() {
        implements_sized::<ParseCompanyFactsOutput>();
    }

    const fn implements_hash<T: Hash>() {}
    #[test]
    const fn should_implement_hash_when_implementing_output_data_trait() {
        implements_hash::<ParseCompanyFactsOutput>();
    }

    const fn implements_partial_eq<T: PartialEq>() {}
    #[test]
    const fn should_implement_partial_eq_when_implementing_output_data_trait() {
        implements_partial_eq::<ParseCompanyFactsOutput>();
    }

    const fn implements_eq<T: Eq>() {}
    #[test]
    const fn should_implement_eq_when_implementing_output_data_trait() {
        implements_eq::<ParseCompanyFactsOutput>();
    }

    const fn implements_partial_ord<T: PartialOrd>() {}
    #[test]
    const fn should_implement_partial_ord_when_implementing_output_data_trait() {
        implements_partial_ord::<ParseCompanyFactsOutput>();
    }

    const fn implements_ord<T: Ord>() {}
    #[test]
    const fn should_implement_ord_when_implementing_output_data_trait() {
        implements_ord::<ParseCompanyFactsOutput>();
    }

    const fn implements_debug<T: Debug>() {}
    #[test]
    const fn should_implement_debug_when_implementing_output_data_trait() {
        implements_debug::<ParseCompanyFactsOutput>();
    }

    const fn implements_clone<T: Clone>() {}
    #[test]
    const fn should_implement_clone_when_implementing_output_data_trait() {
        implements_clone::<ParseCompanyFactsOutput>();
    }

    const fn implements_unpin<T: Unpin>() {}
    #[test]
    const fn should_implement_unpin_when_implementing_output_data_trait() {
        implements_unpin::<ParseCompanyFactsOutput>();
    }
}
