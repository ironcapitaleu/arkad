//! # Store Filing Documents Input
//!
//! Provides the [`StoreFilingDocumentsInput`] fed into the
//! [`StoreFilingDocuments`](crate::implementations::states::extract::store_filing_documents::StoreFilingDocuments)
//! state, together with its updater and builder.

use std::fmt;
use std::sync::Arc;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::filing_documents::FilingDocumentResponses;
use crate::shared::filing_ref::FilingRef;
use crate::shared::url::Url;
use crate::shared::user_agent::UserAgent;
use crate::traits::state_machine::state::StateData;

/// Input data for the [`StoreFilingDocuments`](super::super::StoreFilingDocuments) state.
///
/// The responses sit behind an [`Arc`], because a `Clone` of a body would copy the whole file.
/// The state takes the responses out of the input, so the field is `None` after a store.
/// `labels_url` is `Some` exactly when the responses hold a label file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreFilingDocumentsInput {
    /// The downloaded files of the filing, or `None` after the state took them.
    pub responses: Option<Arc<FilingDocumentResponses>>,
    /// The filing the files belong to.
    pub filing: FilingRef,
    /// The URL the instance download sent, before any redirects.
    pub instance_url: Url,
    /// The URL the label file download sent, before any redirects.
    pub labels_url: Option<Url>,
    /// The `User-Agent` the downloads sent.
    pub user_agent: UserAgent,
}

impl StoreFilingDocumentsInput {
    /// Creates a new [`StoreFilingDocumentsInput`] from the downloaded files, their filing, and
    /// the URLs and the `User-Agent` the downloads sent.
    #[must_use]
    pub const fn new(
        responses: Arc<FilingDocumentResponses>,
        filing: FilingRef,
        instance_url: Url,
        labels_url: Option<Url>,
        user_agent: UserAgent,
    ) -> Self {
        Self {
            responses: Some(responses),
            filing,
            instance_url,
            labels_url,
            user_agent,
        }
    }
}

impl StateData for StoreFilingDocumentsInput {
    /// Replaces each field that the updater sets.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(responses) = updates.responses {
            self.responses = Some(responses);
        }
        if let Some(filing) = updates.filing {
            self.filing = filing;
        }
        if let Some(instance_url) = updates.instance_url {
            self.instance_url = instance_url;
        }
        if let Some(labels_url) = updates.labels_url {
            self.labels_url = labels_url;
        }
        if let Some(user_agent) = updates.user_agent {
            self.user_agent = user_agent;
        }
        Ok(())
    }
}

impl SMStateData for StoreFilingDocumentsInput {
    type UpdateType = StoreFilingDocumentsInputUpdater;

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

/// Serializes the responses without their body bytes, or `null` after the state took them.
impl Serialize for StoreFilingDocumentsInput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("StoreFilingDocumentsInput", 5)?;
        state.serialize_field("responses", &self.responses.as_deref())?;
        state.serialize_field("filing", &self.filing)?;
        state.serialize_field("instance_url", &self.instance_url)?;
        state.serialize_field("labels_url", &self.labels_url)?;
        state.serialize_field("user_agent", &self.user_agent)?;
        state.end()
    }
}

impl fmt::Display for StoreFilingDocumentsInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tFiling: {}\n\tInstance URL: {}\n\tLabels URL: ",
            self.filing, self.instance_url
        )?;
        match &self.labels_url {
            Some(labels_url) => write!(f, "{labels_url}")?,
            None => write!(f, "None")?,
        }
        write!(f, "\n\tResponses: ")?;
        match &self.responses {
            Some(responses) => write!(f, "{responses}"),
            None => write!(f, "None"),
        }
    }
}

/// Updater for [`StoreFilingDocumentsInput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone)]
pub struct StoreFilingDocumentsInputUpdater {
    /// The new downloaded files.
    pub responses: Option<Arc<FilingDocumentResponses>>,
    /// The new filing.
    pub filing: Option<FilingRef>,
    /// The new instance URL.
    pub instance_url: Option<Url>,
    /// The new label file URL. `Some(None)` marks a filing without a label file.
    pub labels_url: Option<Option<Url>>,
    /// The new `User-Agent`.
    pub user_agent: Option<UserAgent>,
}

impl StoreFilingDocumentsInputUpdater {
    /// Returns a [`StoreFilingDocumentsInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> StoreFilingDocumentsInputUpdaterBuilder {
        StoreFilingDocumentsInputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`StoreFilingDocumentsInputUpdater`].
#[derive(Debug)]
pub struct StoreFilingDocumentsInputUpdaterBuilder {
    updater: StoreFilingDocumentsInputUpdater,
}

impl StoreFilingDocumentsInputUpdaterBuilder {
    /// Creates a [`StoreFilingDocumentsInputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            updater: StoreFilingDocumentsInputUpdater {
                responses: None,
                filing: None,
                instance_url: None,
                labels_url: None,
                user_agent: None,
            },
        }
    }

    /// Sets the downloaded files.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn responses(mut self, responses: Arc<FilingDocumentResponses>) -> Self {
        self.updater.responses = Some(responses);
        self
    }

    /// Sets the filing.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn filing(mut self, filing: FilingRef) -> Self {
        self.updater.filing = Some(filing);
        self
    }

    /// Sets the instance URL.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn instance_url(mut self, instance_url: Url) -> Self {
        self.updater.instance_url = Some(instance_url);
        self
    }

    /// Sets the label file URL. `None` marks a filing without a label file.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn labels_url(mut self, labels_url: Option<Url>) -> Self {
        self.updater.labels_url = Some(labels_url);
        self
    }

    /// Sets the `User-Agent`.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn user_agent(mut self, user_agent: UserAgent) -> Self {
        self.updater.user_agent = Some(user_agent);
        self
    }

    /// Builds the [`StoreFilingDocumentsInputUpdater`].
    #[must_use]
    pub fn build(self) -> StoreFilingDocumentsInputUpdater {
        self.updater
    }
}

impl Default for StoreFilingDocumentsInputUpdaterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::hash::Hash;

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::content_type::ContentType;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::headers::Headers;
    use crate::shared::response::implementations::sec_response::{SecResponse, XmlBody};
    use crate::shared::status_code::StatusCode;

    const INSTANCE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_htm.xml";
    const LABELS_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_lab.xml";

    fn url(raw_url: &str) -> Url {
        raw_url
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn filing(accession_number: &str) -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new(accession_number).expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenK, false),
            NaiveDate::from_ymd_opt(2026, 9, 8)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    fn xml_response(raw_url: &str, body: &[u8]) -> SecResponse<XmlBody> {
        SecResponse::from_parts(
            url(raw_url),
            Headers::new(HashMap::new()),
            ContentType::Xml,
            StatusCode::Ok,
            body.to_vec(),
        )
        .expect(
            "Given status 200 and an XML content type, building the response should always succeed",
        )
    }

    fn responses(instance_body: &[u8]) -> Arc<FilingDocumentResponses> {
        Arc::new(FilingDocumentResponses::new(
            xml_response(INSTANCE_URL, instance_body),
            Some(xml_response(LABELS_URL, b"<labels/>")),
        ))
    }

    fn user_agent() -> UserAgent {
        UserAgent::new("arkad contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed")
    }

    fn sample_input(instance_body: &[u8]) -> StoreFilingDocumentsInput {
        StoreFilingDocumentsInput::new(
            responses(instance_body),
            filing("0000003545-26-000047"),
            url(INSTANCE_URL),
            Some(url(LABELS_URL)),
            user_agent(),
        )
    }

    #[test]
    fn should_serialize_the_instance_without_its_body_for_store_filing_documents_input() {
        let input = sample_input(b"<xbrl/>");

        let expected_result = vec![
            "body_sha256",
            "content_type",
            "headers",
            "status_code",
            "url",
        ];

        let value = serde_json::to_value(&input)
            .expect("Given responses of strings and digests, serialization should always succeed");
        let result: Vec<&str> = value["responses"]["instance"]
            .as_object()
            .map(|fields| fields.keys().map(String::as_str).collect())
            .unwrap_or_default();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_serialize_a_null_responses_field_when_the_state_took_the_responses() {
        let mut input = sample_input(b"<xbrl/>");
        input.responses = None;

        let expected_result = Some(serde_json::Value::Null);

        let value = serde_json::to_value(&input).expect(
            "Given an input of strings and no responses, serialization should always succeed",
        );
        let result = value.get("responses").cloned();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_equal_when_two_inputs_have_different_instance_bodies() {
        let input = sample_input(b"<xbrl/>");
        let other = sample_input(b"<xbrl></xbrl>");

        let expected_result = false;

        let result = input == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_responses_when_update_contains_responses() {
        let mut input = sample_input(b"<xbrl/>");
        let update = StoreFilingDocumentsInputUpdater::builder()
            .responses(responses(b"<xbrl></xbrl>"))
            .build();

        let expected_result = &sample_input(b"<xbrl></xbrl>");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_filing_when_update_contains_filing() {
        let mut input = sample_input(b"<xbrl/>");
        let update = StoreFilingDocumentsInputUpdater::builder()
            .filing(filing("0000003545-26-000048"))
            .build();

        let expected_result = filing("0000003545-26-000048");

        SMStateData::update_state(&mut input, update);
        let result = input.filing;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_instance_url_when_update_contains_instance_url() {
        let mut input = sample_input(b"<xbrl/>");
        let update = StoreFilingDocumentsInputUpdater::builder()
            .instance_url(url(LABELS_URL))
            .build();

        let expected_result = url(LABELS_URL);

        SMStateData::update_state(&mut input, update);
        let result = input.instance_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_clear_labels_url_when_update_sets_no_label_file() {
        let mut input = sample_input(b"<xbrl/>");
        let update = StoreFilingDocumentsInputUpdater::builder()
            .labels_url(None)
            .build();

        let expected_result = None;

        SMStateData::update_state(&mut input, update);
        let result = input.labels_url;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_user_agent_when_update_contains_user_agent() {
        let mut input = sample_input(b"<xbrl/>");
        let new_user_agent = UserAgent::new("other contact@example.com")
            .expect("Given a hardcoded SEC-format user agent, validation should always succeed");
        let update = StoreFilingDocumentsInputUpdater::builder()
            .user_agent(new_user_agent.clone())
            .build();

        let expected_result = new_user_agent;

        SMStateData::update_state(&mut input, update);
        let result = input.user_agent;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_input_unchanged_when_update_is_empty() {
        let mut input = sample_input(b"<xbrl/>");
        let update = StoreFilingDocumentsInputUpdater::builder().build();

        let expected_result = &sample_input(b"<xbrl/>");

        SMStateData::update_state(&mut input, update);
        let result = input.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_the_filing_the_urls_and_the_responses_when_the_input_holds_responses() {
        let input = sample_input(b"<xbrl/>");

        let expected_result = format!(
            "\tFiling: 0000003545-26-000047\n\tInstance URL: {INSTANCE_URL}\n\tLabels URL: {LABELS_URL}\n\tResponses: Instance: 200 {INSTANCE_URL}, Label File: 200 {LABELS_URL}"
        );

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_none_when_the_input_has_no_label_file_and_no_responses() {
        let mut input = sample_input(b"<xbrl/>");
        input.responses = None;
        input.labels_url = None;

        let expected_result = format!(
            "\tFiling: 0000003545-26-000047\n\tInstance URL: {INSTANCE_URL}\n\tLabels URL: None\n\tResponses: None"
        );

        let result = input.to_string();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_filing_documents_input() {
        implements_value_traits::<StoreFilingDocumentsInput>();
    }
}
