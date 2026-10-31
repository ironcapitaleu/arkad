//! # Store SEC Response Input
//!
//! Provides the [`StoreSecResponseInput`] fed into the
//! [`StoreSecResponse`](crate::implementations::states::extract::store_sec_response::StoreSecResponse)
//! state, along with its updater and builder.
//!
//! It carries the response to store, the URL and the [`UserAgent`] the fetch sent, and the
//! [`IngestionMode`] of the run.

use std::fmt;
use std::sync::Arc;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::StateData as SMStateData;
use storage::implementations::repository::filesystem::IngestionMode;

use crate::error::State as StateError;
use crate::shared::http_client::HasResponse;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`StoreSecResponse`](super::super::StoreSecResponse) state.
///
/// Bundles the response to a request of kind `E` with the URL and the [`UserAgent`] the fetch
/// sent, which the response does not record. The [`IngestionMode`] comes from the run, because one
/// kind can run as a backfill and as an incremental fetch.
///
/// The response sits in an [`Arc`], so the transition from
/// [`ExecuteSecRequest`](crate::implementations::states::extract::execute_sec_request::ExecuteSecRequest)
/// moves it without a copy.
#[derive(Debug, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreSecResponseInput<E: HasResponse> {
    /// The response whose raw body the state stores.
    pub response: Arc<E::Response>,
    /// The URL the fetch sent, before any redirects.
    pub requested_url: Url,
    /// The `User-Agent` the fetch sent.
    pub user_agent: UserAgent,
    /// The ingestion mode that the metadata of the stored document records.
    pub ingestion_mode: IngestionMode,
}

impl<E: HasResponse> StoreSecResponseInput<E> {
    /// Creates a new [`StoreSecResponseInput`] from a response, the URL and the `User-Agent` the
    /// fetch sent, and the ingestion mode of the run.
    ///
    /// The response is either owned or already in an [`Arc`].
    #[must_use]
    pub fn new(
        response: impl Into<Arc<E::Response>>,
        requested_url: Url,
        user_agent: UserAgent,
        ingestion_mode: IngestionMode,
    ) -> Self {
        Self {
            response: response.into(),
            requested_url,
            user_agent,
            ingestion_mode,
        }
    }

    /// Returns a reference to the response.
    #[must_use]
    pub fn response(&self) -> &E::Response {
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

    /// Returns the ingestion mode of the run.
    #[must_use]
    pub const fn ingestion_mode(&self) -> IngestionMode {
        self.ingestion_mode
    }
}

// A derive bounds `E::Response: Clone`, but cloning the `Arc` needs no such bound.
impl<E: HasResponse> Clone for StoreSecResponseInput<E> {
    fn clone(&self) -> Self {
        Self {
            response: Arc::clone(&self.response),
            requested_url: self.requested_url.clone(),
            user_agent: self.user_agent.clone(),
            ingestion_mode: self.ingestion_mode,
        }
    }
}

// Serde implements `Serialize` for `Arc` only with the `rc` feature, so this impl serializes the
// response behind it.
impl<E: HasResponse> Serialize for StoreSecResponseInput<E> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StoreSecResponseInput", 4)?;
        state.serialize_field("response", self.response.as_ref())?;
        state.serialize_field("requested_url", &self.requested_url)?;
        state.serialize_field("user_agent", &self.user_agent)?;
        state.serialize_field("ingestion_mode", &self.ingestion_mode)?;
        state.end()
    }
}

impl<E: HasResponse> StateData for StoreSecResponseInput<E> {
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
        if let Some(ingestion_mode) = updates.ingestion_mode {
            self.ingestion_mode = ingestion_mode;
        }
        Ok(())
    }
}

impl<E: HasResponse> SMStateData for StoreSecResponseInput<E> {
    type UpdateType = StoreSecResponseInputUpdater<E>;

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

impl<E: HasResponse> fmt::Display for StoreSecResponseInput<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tRequested URL: {}\n\tResponse: {}",
            self.requested_url, self.response
        )
    }
}

/// Updater for modifying [`StoreSecResponseInput`].
///
/// Fields set to `None` are left unchanged when the updater is applied.
#[derive(Debug, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct StoreSecResponseInputUpdater<E: HasResponse> {
    /// Optional new value for the response.
    pub response: Option<Arc<E::Response>>,
    /// Optional new value for the requested URL.
    pub requested_url: Option<Url>,
    /// Optional new value for the `User-Agent`.
    pub user_agent: Option<UserAgent>,
    /// Optional new value for the ingestion mode.
    pub ingestion_mode: Option<IngestionMode>,
}

// A derive bounds `E::Response: Clone`, but cloning the `Arc` needs no such bound.
impl<E: HasResponse> Clone for StoreSecResponseInputUpdater<E> {
    fn clone(&self) -> Self {
        Self {
            response: self.response.clone(),
            requested_url: self.requested_url.clone(),
            user_agent: self.user_agent.clone(),
            ingestion_mode: self.ingestion_mode,
        }
    }
}

impl<E: HasResponse> StoreSecResponseInputUpdater<E> {
    /// Creates a new builder for constructing [`StoreSecResponseInputUpdater`] instances.
    #[must_use]
    pub const fn builder() -> StoreSecResponseInputUpdaterBuilder<E> {
        StoreSecResponseInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreSecResponseInputUpdater`].
pub struct StoreSecResponseInputUpdaterBuilder<E: HasResponse> {
    response: Option<Arc<E::Response>>,
    requested_url: Option<Url>,
    user_agent: Option<UserAgent>,
    ingestion_mode: Option<IngestionMode>,
}

impl<E: HasResponse> StoreSecResponseInputUpdaterBuilder<E> {
    /// Creates a new [`StoreSecResponseInputUpdaterBuilder`] with all fields initialized to `None`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            response: None,
            requested_url: None,
            user_agent: None,
            ingestion_mode: None,
        }
    }

    /// Sets the response field.
    #[must_use]
    pub fn response(mut self, response: impl Into<Arc<E::Response>>) -> Self {
        self.response = Some(response.into());
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

    /// Sets the ingestion mode field.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn ingestion_mode(mut self, ingestion_mode: IngestionMode) -> Self {
        self.ingestion_mode = Some(ingestion_mode);
        self
    }

    /// Builds the [`StoreSecResponseInputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreSecResponseInputUpdater<E> {
        StoreSecResponseInputUpdater {
            response: self.response,
            requested_url: self.requested_url,
            user_agent: self.user_agent,
            ingestion_mode: self.ingestion_mode,
        }
    }
}

impl<E: HasResponse> Default for StoreSecResponseInputUpdaterBuilder<E> {
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
    use crate::implementations::states::extract::store_company_facts::StoreCompanyFactsInput;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::request::implementations::sec_request::CompanyFacts;
    use crate::shared::response::implementations::sec_response::SecResponse;
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
            IngestionMode::Backfill,
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
        let updater = StoreSecResponseInputUpdater::<CompanyFacts>::builder()
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
        let updater = StoreSecResponseInputUpdater::<CompanyFacts>::builder()
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
        let updater = StoreSecResponseInputUpdater::<CompanyFacts>::builder()
            .user_agent(new_user_agent.clone())
            .build();

        let expected_result = &new_user_agent;

        StateData::update_state(&mut input, updater)
            .expect("Given a non-empty update, updating the input should always succeed");
        let result = input.user_agent();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_ingestion_mode_when_updater_contains_ingestion_mode() {
        let mut input = sample_input();
        let updater = StoreSecResponseInputUpdater::<CompanyFacts>::builder()
            .ingestion_mode(IngestionMode::Incremental)
            .build();

        let expected_result = IngestionMode::Incremental;

        StateData::update_state(&mut input, updater)
            .expect("Given a non-empty update, updating the input should always succeed");
        let result = input.ingestion_mode();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_updater_is_empty() {
        let mut input = sample_input();
        let updater = StoreSecResponseInputUpdater::<CompanyFacts>::builder().build();

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
