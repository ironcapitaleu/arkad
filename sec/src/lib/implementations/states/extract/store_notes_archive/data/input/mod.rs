//! # Store Notes Archive Input
//!
//! Provides the [`StoreNotesArchiveInput`] fed into the
//! [`StoreNotesArchive`](crate::implementations::states::extract::store_notes_archive::StoreNotesArchive)
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
use crate::shared::notes_period::NotesPeriod;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`StoreNotesArchive`](super::super::StoreNotesArchive) state.
///
/// The response sits behind an [`Arc`], because a `Clone` of the body would copy the whole file.
/// The state takes the response out of the input, so the field is `None` after a successful
/// store. Equality, ordering, and hashing use the SHA-256 digest of the body, not the bytes.
#[derive(Debug, Clone)]
pub struct StoreNotesArchiveInput {
    /// The downloaded Notes file, or `None` after the state took it.
    pub response: Option<Arc<SecResponse<ZipBody>>>,
    /// The period of the Notes file.
    pub notes_period: NotesPeriod,
    /// The URL the download sent, before any redirects.
    pub requested_url: Url,
    /// The `User-Agent` the download sent.
    pub user_agent: UserAgent,
}

impl StoreNotesArchiveInput {
    /// Creates a new [`StoreNotesArchiveInput`] from a downloaded Notes file, its period, and the
    /// URL and the `User-Agent` the download sent.
    #[must_use]
    pub const fn new(
        response: Arc<SecResponse<ZipBody>>,
        notes_period: NotesPeriod,
        requested_url: Url,
        user_agent: UserAgent,
    ) -> Self {
        Self {
            response: Some(response),
            notes_period,
            requested_url,
            user_agent,
        }
    }

    /// Returns the fields that equality, ordering, and hashing compare.
    fn comparison_key(&self) -> (Option<&Sha256Digest>, NotesPeriod, &Url, &UserAgent) {
        (
            self.response
                .as_deref()
                .map(SecResponse::<ZipBody>::body_digest),
            self.notes_period,
            &self.requested_url,
            &self.user_agent,
        )
    }
}

impl PartialEq for StoreNotesArchiveInput {
    fn eq(&self, other: &Self) -> bool {
        self.comparison_key() == other.comparison_key()
    }
}

impl Eq for StoreNotesArchiveInput {}

impl PartialOrd for StoreNotesArchiveInput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StoreNotesArchiveInput {
    fn cmp(&self, other: &Self) -> Ordering {
        self.comparison_key().cmp(&other.comparison_key())
    }
}

impl Hash for StoreNotesArchiveInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.comparison_key().hash(state);
    }
}

impl StateData for StoreNotesArchiveInput {
    /// Replaces each field that the updater sets.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.response = Some(response);
        }
        if let Some(notes_period) = updates.notes_period {
            self.notes_period = notes_period;
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

impl SMStateData for StoreNotesArchiveInput {
    type UpdateType = StoreNotesArchiveInputUpdater;

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

/// Serializes the response without its body bytes, or `null` after the state took it.
impl Serialize for StoreNotesArchiveInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StoreNotesArchiveInput", 4)?;
        state.serialize_field("response", &self.response.as_deref())?;
        state.serialize_field("notes_period", &self.notes_period)?;
        state.serialize_field("requested_url", &self.requested_url)?;
        state.serialize_field("user_agent", &self.user_agent)?;
        state.end()
    }
}

impl fmt::Display for StoreNotesArchiveInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tNotes Period: {}\n\tRequested URL: {}\n\tResponse: ",
            self.notes_period, self.requested_url
        )?;
        match &self.response {
            Some(response) => write!(
                f,
                "{}, Body SHA-256: {}",
                response.url(),
                response.body_digest()
            ),
            None => write!(f, "None"),
        }
    }
}

/// Updater for [`StoreNotesArchiveInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone)]
pub struct StoreNotesArchiveInputUpdater {
    /// The new downloaded Notes file.
    pub response: Option<Arc<SecResponse<ZipBody>>>,
    /// The new Notes period.
    pub notes_period: Option<NotesPeriod>,
    /// The new requested URL.
    pub requested_url: Option<Url>,
    /// The new `User-Agent`.
    pub user_agent: Option<UserAgent>,
}

impl StoreNotesArchiveInputUpdater {
    /// Returns a [`StoreNotesArchiveInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> StoreNotesArchiveInputUpdaterBuilder {
        StoreNotesArchiveInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreNotesArchiveInputUpdater`].
#[derive(Debug, Default)]
pub struct StoreNotesArchiveInputUpdaterBuilder {
    response: Option<Arc<SecResponse<ZipBody>>>,
    notes_period: Option<NotesPeriod>,
    requested_url: Option<Url>,
    user_agent: Option<UserAgent>,
}

impl StoreNotesArchiveInputUpdaterBuilder {
    /// Creates a [`StoreNotesArchiveInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            response: None,
            notes_period: None,
            requested_url: None,
            user_agent: None,
        }
    }

    /// Sets the downloaded Notes file.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn response(mut self, response: Arc<SecResponse<ZipBody>>) -> Self {
        self.response = Some(response);
        self
    }

    /// Sets the Notes period.
    #[must_use]
    pub const fn notes_period(mut self, notes_period: NotesPeriod) -> Self {
        self.notes_period = Some(notes_period);
        self
    }

    /// Sets the requested URL.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn requested_url(mut self, requested_url: Url) -> Self {
        self.requested_url = Some(requested_url);
        self
    }

    /// Sets the `User-Agent`.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn user_agent(mut self, user_agent: UserAgent) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Builds the [`StoreNotesArchiveInputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreNotesArchiveInputUpdater {
        StoreNotesArchiveInputUpdater {
            response: self.response,
            notes_period: self.notes_period,
            requested_url: self.requested_url,
            user_agent: self.user_agent,
        }
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

    fn notes_period(period: &str) -> NotesPeriod {
        NotesPeriod::new(period)
            .expect("Given a hardcoded published period, parsing should always succeed")
    }

    fn notes_url(file_name: &str) -> Url {
        format!(
            "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/{file_name}"
        )
        .parse()
        .expect("Given a hardcoded valid URL prefix, parsing should always succeed")
    }

    fn notes_file_response(body: &[u8]) -> Arc<SecResponse<ZipBody>> {
        let headers = Headers::new(HashMap::from([(
            "content-type".to_string(),
            "application/zip".to_string(),
        )]));
        let response = SecResponse::<ZipBody>::from_parts(
            notes_url("2024q1_notes.zip"),
            headers,
            ContentType::Zip,
            StatusCode::Ok,
            body.to_vec(),
        )
        .expect(
            "Given status 200 and a ZIP content type, building the response should always succeed",
        );
        Arc::new(response)
    }

    fn user_agent() -> UserAgent {
        UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed")
    }

    fn sample_input(body: &[u8]) -> StoreNotesArchiveInput {
        StoreNotesArchiveInput::new(
            notes_file_response(body),
            notes_period("2024q1"),
            notes_url("2024q1_notes.zip"),
            user_agent(),
        )
    }

    #[test]
    fn should_serialize_the_response_without_its_body_for_store_notes_archive_input() {
        let input = sample_input(b"PK");

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
    fn should_serialize_a_null_response_when_the_state_took_the_response() {
        let mut input = sample_input(b"PK");
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
        let input = sample_input(b"PK");
        let other = sample_input(b"PK");

        let expected_result = true;

        let result = input == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_equal_when_two_inputs_have_different_bodies() {
        let input = sample_input(b"PK");
        let other = sample_input(b"PK!");

        let expected_result = false;

        let result = input == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_response_when_update_contains_response() {
        let mut input = sample_input(b"PK");
        let update = StoreNotesArchiveInputUpdater::builder()
            .response(notes_file_response(b"PK!"))
            .build();

        let expected_result = &sample_input(b"PK!");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_notes_period_when_update_contains_notes_period() {
        let mut input = sample_input(b"PK");
        let update = StoreNotesArchiveInputUpdater::builder()
            .notes_period(notes_period("2025_10"))
            .build();

        let expected_result = notes_period("2025_10");

        SMStateData::update_state(&mut input, update);
        let result = input.notes_period;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_requested_url_when_update_contains_requested_url() {
        let mut input = sample_input(b"PK");
        let update = StoreNotesArchiveInputUpdater::builder()
            .requested_url(notes_url("2025_10_notes.zip"))
            .build();

        let expected_result = notes_url("2025_10_notes.zip");

        SMStateData::update_state(&mut input, update);
        let result = input.requested_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_user_agent_when_update_contains_user_agent() {
        let mut input = sample_input(b"PK");
        let new_user_agent = UserAgent::new("other contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        let update = StoreNotesArchiveInputUpdater::builder()
            .user_agent(new_user_agent.clone())
            .build();

        let expected_result = new_user_agent;

        SMStateData::update_state(&mut input, update);
        let result = input.user_agent;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = sample_input(b"PK");
        let update = StoreNotesArchiveInputUpdater::builder().build();

        let expected_result = &sample_input(b"PK");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_response_url_and_digest_when_the_input_holds_a_response() {
        let mut input = sample_input(b"PK");
        input.requested_url = notes_url("2024q1_requested.zip");

        let expected_result = format!(
            "\tNotes Period: 2024q1\n\tRequested URL: https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_requested.zip\n\tResponse: https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip, Body SHA-256: {}",
            Sha256Digest::of(b"PK")
        );

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_none_as_the_response_when_the_state_took_the_response() {
        let mut input = sample_input(b"PK");
        input.response = None;

        let expected_result = "\tNotes Period: 2024q1\n\tRequested URL: https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip\n\tResponse: None";

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_notes_archive_input() {
        implements_value_traits::<StoreNotesArchiveInput>();
    }
}
