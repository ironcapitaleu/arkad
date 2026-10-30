//! # Store Daily Index Input
//!
//! Provides the [`StoreDailyIndexInput`] fed into the
//! [`StoreDailyIndex`](crate::implementations::states::extract::store_daily_index::StoreDailyIndex)
//! state, together with its updater and builder.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::StateData as SMStateData;
use storage::Sha256Digest;

use crate::error::State as StateError;
use crate::shared::index_date::IndexDate;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`StoreDailyIndex`](super::super::StoreDailyIndex) state.
///
/// The response is `None` if the day had no filings, as in
/// [`DownloadDailyIndexOutput`](crate::implementations::states::extract::download_daily_index::DownloadDailyIndexOutput).
/// Equality, ordering, and hashing compare the SHA-256 digest of the body, not its bytes, and the
/// other fields.
#[derive(Debug, Clone)]
pub struct StoreDailyIndexInput {
    /// The downloaded daily index, or `None` if the day had no filings.
    pub response: Option<Arc<SecResponse<ZipBody>>>,
    /// The date of the daily index.
    pub index_date: IndexDate,
    /// The URL the download sent, before any redirects.
    pub requested_url: Url,
    /// The `User-Agent` the download sent.
    pub user_agent: UserAgent,
}

impl StoreDailyIndexInput {
    /// Creates a new [`StoreDailyIndexInput`] from a downloaded daily index, its date, and the URL
    /// and the `User-Agent` the download sent.
    ///
    /// A `response` of `None` marks a day without filings.
    #[must_use]
    pub const fn new(
        response: Option<Arc<SecResponse<ZipBody>>>,
        index_date: IndexDate,
        requested_url: Url,
        user_agent: UserAgent,
    ) -> Self {
        Self {
            response,
            index_date,
            requested_url,
            user_agent,
        }
    }

    /// Returns the fields that equality, ordering, and hashing compare.
    fn comparison_key(&self) -> (Option<&Sha256Digest>, IndexDate, &Url, &UserAgent) {
        (
            self.response
                .as_deref()
                .map(SecResponse::<ZipBody>::body_digest),
            self.index_date,
            &self.requested_url,
            &self.user_agent,
        )
    }
}

impl PartialEq for StoreDailyIndexInput {
    fn eq(&self, other: &Self) -> bool {
        self.comparison_key() == other.comparison_key()
    }
}

impl Eq for StoreDailyIndexInput {}

impl PartialOrd for StoreDailyIndexInput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StoreDailyIndexInput {
    fn cmp(&self, other: &Self) -> Ordering {
        self.comparison_key().cmp(&other.comparison_key())
    }
}

impl Hash for StoreDailyIndexInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.comparison_key().hash(state);
    }
}

impl StateData for StoreDailyIndexInput {
    /// Replaces each field that the updater sets.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.response = response;
        }
        if let Some(index_date) = updates.index_date {
            self.index_date = index_date;
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

impl SMStateData for StoreDailyIndexInput {
    type UpdateType = StoreDailyIndexInputUpdater;

    fn state(&self) -> &Self {
        self
    }

    /// Delegates to the SEC [`StateData::update_state`] implementation.
    ///
    /// # Panics
    ///
    /// Panics if the fallible SEC update returns an error.
    fn update_state(&mut self, updates: Self::UpdateType) {
        if let Err(e) = <Self as StateData>::update_state(self, updates) {
            panic!("StateData::update_state failed: {e}")
        }
    }
}

/// Serializes the response without its body bytes, or `null` for a day without filings.
impl Serialize for StoreDailyIndexInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StoreDailyIndexInput", 4)?;
        state.serialize_field("response", &self.response.as_deref())?;
        state.serialize_field("index_date", &self.index_date)?;
        state.serialize_field("requested_url", &self.requested_url)?;
        state.serialize_field("user_agent", &self.user_agent)?;
        state.end()
    }
}

impl fmt::Display for StoreDailyIndexInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tIndex Date: {}\n\tRequested URL: {}\n\tResponse: ",
            self.index_date, self.requested_url
        )?;
        match &self.response {
            Some(response) => write!(
                f,
                "{}, Body SHA-256: {}",
                response.url(),
                response.body_digest()
            ),
            None => write!(f, "No filings on this day"),
        }
    }
}

/// Updater for [`StoreDailyIndexInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone)]
pub struct StoreDailyIndexInputUpdater {
    /// The new response. `Some(None)` marks a day without filings.
    pub response: Option<Option<Arc<SecResponse<ZipBody>>>>,
    /// The new index date.
    pub index_date: Option<IndexDate>,
    /// The new requested URL.
    pub requested_url: Option<Url>,
    /// The new `User-Agent`.
    pub user_agent: Option<UserAgent>,
}

impl StoreDailyIndexInputUpdater {
    /// Returns a [`StoreDailyIndexInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> StoreDailyIndexInputUpdaterBuilder {
        StoreDailyIndexInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreDailyIndexInputUpdater`].
#[derive(Debug)]
pub struct StoreDailyIndexInputUpdaterBuilder {
    updater: StoreDailyIndexInputUpdater,
}

impl StoreDailyIndexInputUpdaterBuilder {
    /// Creates a [`StoreDailyIndexInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            updater: StoreDailyIndexInputUpdater {
                response: None,
                index_date: None,
                requested_url: None,
                user_agent: None,
            },
        }
    }

    /// Sets the response. `None` marks a day without filings.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn response(mut self, response: Option<Arc<SecResponse<ZipBody>>>) -> Self {
        self.updater.response = Some(response);
        self
    }

    /// Sets the index date.
    #[must_use]
    pub const fn index_date(mut self, index_date: IndexDate) -> Self {
        self.updater.index_date = Some(index_date);
        self
    }

    /// Sets the requested URL.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn requested_url(mut self, requested_url: Url) -> Self {
        self.updater.requested_url = Some(requested_url);
        self
    }

    /// Sets the `User-Agent`.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn user_agent(mut self, user_agent: UserAgent) -> Self {
        self.updater.user_agent = Some(user_agent);
        self
    }

    /// Builds the [`StoreDailyIndexInputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreDailyIndexInputUpdater {
        self.updater
    }
}

impl Default for StoreDailyIndexInputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Debug;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::content_type::ContentType;
    use crate::shared::headers::Headers;
    use crate::shared::status_code::StatusCode;

    fn index_date(date: &str) -> IndexDate {
        IndexDate::new(date).expect("Given a hardcoded past date, parsing should always succeed")
    }

    fn daily_index_url(file_name: &str) -> Url {
        format!("https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/{file_name}")
            .parse()
            .expect("Given a hardcoded valid URL prefix, parsing should always succeed")
    }

    fn daily_index_response(body: &[u8]) -> Arc<SecResponse<ZipBody>> {
        let response = SecResponse::<ZipBody>::from_parts(
            daily_index_url("form.20261027.idx"),
            Headers::new(HashMap::new()),
            ContentType::OctetStream,
            StatusCode::Ok,
            body.to_vec(),
        )
        .expect(
            "Given status 200 and an octet stream, building the response should always succeed",
        );
        Arc::new(response)
    }

    fn user_agent() -> UserAgent {
        UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed")
    }

    fn sample_input(body: &[u8]) -> StoreDailyIndexInput {
        StoreDailyIndexInput::new(
            Some(daily_index_response(body)),
            index_date("2026-10-27"),
            daily_index_url("form.20261027.idx"),
            user_agent(),
        )
    }

    #[test]
    fn should_serialize_the_response_without_its_body_for_store_daily_index_input() {
        let input = sample_input(b"Form Type");

        let expected_result = vec![
            "body_sha256",
            "content_type",
            "headers",
            "status_code",
            "url",
        ];

        let value = serde_json::to_value(&input).expect(
            "Given a response of strings and a digest, serialization should always succeed",
        );
        let result: Vec<&str> = value["response"]
            .as_object()
            .map(|fields| fields.keys().map(String::as_str).collect())
            .unwrap_or_default();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_a_null_response_for_a_day_without_filings() {
        let mut input = sample_input(b"Form Type");
        input.response = None;

        let expected_result = Some(serde_json::Value::Null);

        let value = serde_json::to_value(&input).expect(
            "Given an input of strings and no response, serialization should always succeed",
        );
        let result = value.get("response").cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_equal_when_two_inputs_have_the_same_body() {
        let input = sample_input(b"Form Type");
        let other = sample_input(b"Form Type");

        let expected_result = true;

        let result = input == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_equal_when_two_inputs_have_different_bodies() {
        let input = sample_input(b"Form Type");
        let other = sample_input(b"Form Type 10-K");

        let expected_result = false;

        let result = input == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_response_when_update_contains_response() {
        let mut input = sample_input(b"Form Type");
        let update = StoreDailyIndexInputUpdater::builder()
            .response(Some(daily_index_response(b"Form Type 10-K")))
            .build();

        let expected_result = &sample_input(b"Form Type 10-K");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_clear_the_response_when_update_marks_a_day_without_filings() {
        let mut input = sample_input(b"Form Type");
        let update = StoreDailyIndexInputUpdater::builder()
            .response(None)
            .build();

        let expected_result = true;

        SMStateData::update_state(&mut input, update);
        let result = input.response.is_none();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_index_date_when_update_contains_index_date() {
        let mut input = sample_input(b"Form Type");
        let update = StoreDailyIndexInputUpdater::builder()
            .index_date(index_date("2026-10-28"))
            .build();

        let expected_result = index_date("2026-10-28");

        SMStateData::update_state(&mut input, update);
        let result = input.index_date;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_requested_url_when_update_contains_requested_url() {
        let mut input = sample_input(b"Form Type");
        let update = StoreDailyIndexInputUpdater::builder()
            .requested_url(daily_index_url("form.20261028.idx"))
            .build();

        let expected_result = daily_index_url("form.20261028.idx");

        SMStateData::update_state(&mut input, update);
        let result = input.requested_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_user_agent_when_update_contains_user_agent() {
        let mut input = sample_input(b"Form Type");
        let new_user_agent = UserAgent::new("other contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        let update = StoreDailyIndexInputUpdater::builder()
            .user_agent(new_user_agent.clone())
            .build();

        let expected_result = new_user_agent;

        SMStateData::update_state(&mut input, update);
        let result = input.user_agent;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = sample_input(b"Form Type");
        let update = StoreDailyIndexInputUpdater::builder().build();

        let expected_result = &sample_input(b"Form Type");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_response_url_and_digest_when_the_input_holds_a_response() {
        let input = sample_input(b"Form Type");

        let expected_result = format!(
            "\tIndex Date: 2026-10-27\n\tRequested URL: https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx\n\tResponse: https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx, Body SHA-256: {}",
            Sha256Digest::of(b"Form Type")
        );

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_no_filings_as_the_response_for_a_day_without_filings() {
        let mut input = sample_input(b"Form Type");
        input.response = None;

        let expected_result = "\tIndex Date: 2026-10-27\n\tRequested URL: https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx\n\tResponse: No filings on this day";

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_daily_index_input() {
        implements_value_traits::<StoreDailyIndexInput>();
    }
}
