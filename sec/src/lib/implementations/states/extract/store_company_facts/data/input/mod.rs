//! # Store Company Facts Input
//!
//! Provides the [`StoreCompanyFactsInput`] fed into the
//! [`StoreCompanyFacts`](crate::implementations::states::extract::store_company_facts::StoreCompanyFacts)
//! state, along with its updater and builder.
//!
//! It carries the [`SecResponse`] to store, and the URL and the [`UserAgent`] the fetch sent.

use std::fmt;

use serde::Serialize;
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::response::implementations::sec_response::SecResponse;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`StoreCompanyFacts`](super::super::StoreCompanyFacts) state.
///
/// Bundles the [`SecResponse`] to store with the URL and the [`UserAgent`] the fetch sent, which
/// the response does not record.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreCompanyFactsInput {
    /// The SEC response whose raw body the state stores.
    pub response: SecResponse,
    /// The URL the fetch sent, before any redirects.
    pub requested_url: Url,
    /// The `User-Agent` the fetch sent.
    pub user_agent: UserAgent,
}

impl StoreCompanyFactsInput {
    /// Creates a new [`StoreCompanyFactsInput`] from a response, and the URL and the `User-Agent`
    /// the fetch sent.
    #[must_use]
    pub const fn new(response: SecResponse, requested_url: Url, user_agent: UserAgent) -> Self {
        Self {
            response,
            requested_url,
            user_agent,
        }
    }

    /// Returns a reference to the SEC response.
    #[must_use]
    pub const fn response(&self) -> &SecResponse {
        &self.response
    }

    /// Returns a reference to the URL the fetch sent.
    #[must_use]
    pub const fn requested_url(&self) -> &Url {
        &self.requested_url
    }

    /// Returns a reference to the `User-Agent` the fetch sent.
    #[must_use]
    pub const fn user_agent(&self) -> &UserAgent {
        &self.user_agent
    }
}

impl StateData for StoreCompanyFactsInput {
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.response = response;
        }
        if let Some(requested_url) = updates.requested_url {
            self.requested_url = requested_url;
        }
        if let Some(user_agent) = updates.user_agent {
            self.user_agent = user_agent;
        }
        Ok(())
    }
}

impl SMStateData for StoreCompanyFactsInput {
    type UpdateType = StoreCompanyFactsInputUpdater;

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

impl fmt::Display for StoreCompanyFactsInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tRequested URL: {}\n\tResponse: {}",
            self.requested_url, self.response
        )
    }
}

/// Updater for modifying [`StoreCompanyFactsInput`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreCompanyFactsInputUpdater {
    /// Optional new value for the SEC response.
    pub response: Option<SecResponse>,
    /// Optional new value for the requested URL.
    pub requested_url: Option<Url>,
    /// Optional new value for the `User-Agent`.
    pub user_agent: Option<UserAgent>,
}

impl StoreCompanyFactsInputUpdater {
    /// Creates a new builder for constructing [`StoreCompanyFactsInputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> StoreCompanyFactsInputUpdaterBuilder {
        StoreCompanyFactsInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreCompanyFactsInputUpdater`].
pub struct StoreCompanyFactsInputUpdaterBuilder {
    response: Option<SecResponse>,
    requested_url: Option<Url>,
    user_agent: Option<UserAgent>,
}

impl StoreCompanyFactsInputUpdaterBuilder {
    /// Creates a new [`StoreCompanyFactsInputUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            response: None,
            requested_url: None,
            user_agent: None,
        }
    }

    /// Sets the SEC response field.
    #[must_use]
    pub fn response(mut self, response: SecResponse) -> Self {
        self.response = Some(response);
        self
    }

    /// Sets the requested URL field.
    #[must_use]
    pub fn requested_url(mut self, requested_url: Url) -> Self {
        self.requested_url = Some(requested_url);
        self
    }

    /// Sets the `User-Agent` field.
    #[must_use]
    pub fn user_agent(mut self, user_agent: UserAgent) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Builds the [`StoreCompanyFactsInputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreCompanyFactsInputUpdater {
        StoreCompanyFactsInputUpdater {
            response: self.response,
            requested_url: self.requested_url,
            user_agent: self.user_agent,
        }
    }
}

impl Default for StoreCompanyFactsInputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::{fmt::Debug, hash::Hash};

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::status_code::StatusCode;

    fn sample_url(path: &str) -> Url {
        format!("https://data.sec.gov/api/xbrl/companyfacts/{path}")
            .parse()
            .expect("Given a hardcoded valid URL prefix, parsing should always succeed")
    }

    fn sample_response(raw_body: &[u8]) -> SecResponse {
        SecResponse::from_parts(
            sample_url("CIK0001067983.json"),
            Headers::new(HashMap::new()),
            ContentType::Json,
            StatusCode::Ok,
            raw_body.to_vec(),
        )
        .expect("Given a hardcoded valid JSON body, building the response should always succeed")
    }

    fn sample_user_agent(name: &str) -> UserAgent {
        UserAgent::new(&format!("{name} contact@example.com"))
            .expect("Given a name and an email, the SEC-format user agent should always validate")
    }

    fn sample_input() -> StoreCompanyFactsInput {
        StoreCompanyFactsInput::new(
            sample_response(b"{}"),
            sample_url("CIK0001067983.json"),
            sample_user_agent("arkad"),
        )
    }

    #[test]
    fn should_return_reference_to_self_when_accessing_state() {
        let input = sample_input();

        let expected_result = &input.clone();

        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_response_when_updater_contains_response() {
        let mut input = sample_input();
        let new_response = sample_response(b"{ \"facts\": {} }");
        let updater = StoreCompanyFactsInputUpdater::builder()
            .response(new_response.clone())
            .build();

        let expected_result = &new_response;

        StateData::update_state(&mut input, updater)
            .expect("Given a non-empty update, updating the input should always succeed");
        let result = input.response();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_requested_url_when_updater_contains_requested_url() {
        let mut input = sample_input();
        let new_url = sample_url("CIK0000320193.json");
        let updater = StoreCompanyFactsInputUpdater::builder()
            .requested_url(new_url.clone())
            .build();

        let expected_result = &new_url;

        StateData::update_state(&mut input, updater)
            .expect("Given a non-empty update, updating the input should always succeed");
        let result = input.requested_url();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_user_agent_when_updater_contains_user_agent() {
        let mut input = sample_input();
        let new_user_agent = sample_user_agent("other");
        let updater = StoreCompanyFactsInputUpdater::builder()
            .user_agent(new_user_agent.clone())
            .build();

        let expected_result = &new_user_agent;

        StateData::update_state(&mut input, updater)
            .expect("Given a non-empty update, updating the input should always succeed");
        let result = input.user_agent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_updater_is_empty() {
        let mut input = sample_input();
        let updater = StoreCompanyFactsInputUpdater::builder().build();

        let expected_result = sample_input();

        StateData::update_state(&mut input, updater)
            .expect("Given an empty update, updating the input should always succeed");
        let result = input;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_requested_url_and_the_response_when_formatted() {
        let input = sample_input();

        let expected_result = "\tRequested URL: https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json\n\tResponse: 200 https://data.sec.gov/api/xbrl/companyfacts/CIK0001067983.json";

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_store_company_facts_input() {
        implements_auto_traits::<StoreCompanyFactsInput>();
    }

    const fn implements_send<T: Send>() {}
    const fn implements_sync<T: Sync>() {}

    #[test]
    const fn should_be_thread_safe_for_store_company_facts_input() {
        implements_send::<StoreCompanyFactsInput>();
        implements_sync::<StoreCompanyFactsInput>();
    }

    const fn implements_value_traits<
        T: Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_company_facts_input() {
        implements_value_traits::<StoreCompanyFactsInput>();
    }
}
