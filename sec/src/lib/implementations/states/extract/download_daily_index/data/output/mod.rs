//! # Download Daily Index Output
//!
//! Provides the [`DownloadDailyIndexOutput`] produced by the
//! [`DownloadDailyIndex`](crate::implementations::states::extract::download_daily_index::DownloadDailyIndex)
//! state, together with its updater and builder.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use state_maschine::prelude::StateData as SMStateData;
use storage::shared::Sha256Digest;

use crate::error::State as StateError;
use crate::shared::response::SecResponse as SecResponseTrait;
use crate::shared::response::implementations::sec_response::{SecResponse, ZipBody};
use crate::traits::state_machine::state::StateData;

/// Output data of the [`DownloadDailyIndex`](super::super::DownloadDailyIndex) state.
///
/// The response is `None` if the day had no filings, so the SEC published no index for it.
/// The response sits behind an [`Arc`], so a clone of the output copies only the pointer.
/// Equality, ordering, and hashing use only the SHA-256 digest of the body.
#[derive(Debug, Clone)]
pub struct DownloadDailyIndexOutput {
    /// The downloaded daily index, or `None` if the day had no filings.
    pub response: Option<Arc<SecResponse<ZipBody>>>,
}

impl DownloadDailyIndexOutput {
    /// Creates a new [`DownloadDailyIndexOutput`] from a downloaded daily index.
    #[must_use]
    pub fn new(response: SecResponse<ZipBody>) -> Self {
        Self {
            response: Some(Arc::new(response)),
        }
    }

    /// Creates a new [`DownloadDailyIndexOutput`] for a day without filings.
    #[must_use]
    pub const fn without_filings() -> Self {
        Self { response: None }
    }

    fn body_digest(&self) -> Option<&Sha256Digest> {
        self.response
            .as_deref()
            .map(SecResponse::<ZipBody>::body_digest)
    }
}

impl PartialEq for DownloadDailyIndexOutput {
    fn eq(&self, other: &Self) -> bool {
        self.body_digest() == other.body_digest()
    }
}

impl Eq for DownloadDailyIndexOutput {}

impl PartialOrd for DownloadDailyIndexOutput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DownloadDailyIndexOutput {
    fn cmp(&self, other: &Self) -> Ordering {
        self.body_digest().cmp(&other.body_digest())
    }
}

impl Hash for DownloadDailyIndexOutput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.body_digest().hash(state);
    }
}

impl StateData for DownloadDailyIndexOutput {
    /// Replaces the response if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.response = response;
        }
        Ok(())
    }
}

impl SMStateData for DownloadDailyIndexOutput {
    type UpdateType = DownloadDailyIndexOutputUpdater;

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

/// Serializes the response without its body bytes.
impl Serialize for DownloadDailyIndexOutput {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("DownloadDailyIndexOutput", 1)?;
        state.serialize_field("response", &self.response.as_deref())?;
        state.end()
    }
}

impl fmt::Display for DownloadDailyIndexOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.response {
            Some(response) => write!(
                f,
                "\tURL: {}, Body SHA-256: {}",
                response.url(),
                response.body_digest()
            ),
            None => write!(f, "\tNo filings on this day"),
        }
    }
}

/// Updater for [`DownloadDailyIndexOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone)]
pub struct DownloadDailyIndexOutputUpdater {
    /// The new response. `Some(None)` marks a day without filings.
    pub response: Option<Option<Arc<SecResponse<ZipBody>>>>,
}

impl DownloadDailyIndexOutputUpdater {
    /// Returns a [`DownloadDailyIndexOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> DownloadDailyIndexOutputUpdaterBuilder {
        DownloadDailyIndexOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`DownloadDailyIndexOutputUpdater`].
#[derive(Debug)]
pub struct DownloadDailyIndexOutputUpdaterBuilder {
    updater: DownloadDailyIndexOutputUpdater,
}

impl DownloadDailyIndexOutputUpdaterBuilder {
    /// Creates a [`DownloadDailyIndexOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            updater: DownloadDailyIndexOutputUpdater { response: None },
        }
    }

    /// Sets the response. `None` marks a day without filings.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn response(mut self, response: Option<Arc<SecResponse<ZipBody>>>) -> Self {
        self.updater.response = Some(response);
        self
    }

    /// Builds the [`DownloadDailyIndexOutputUpdater`].
    #[must_use]
    pub fn build(self) -> DownloadDailyIndexOutputUpdater {
        self.updater
    }
}

impl Default for DownloadDailyIndexOutputUpdaterBuilder {
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
    use crate::shared::url::Url;

    fn daily_index_response(body: &[u8]) -> SecResponse<ZipBody> {
        let url: Url = "https://www.sec.gov/Archives/edgar/daily-index/2026/QTR4/form.20261027.idx"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        SecResponse::<ZipBody>::from_parts(
            url,
            Headers::new(HashMap::new()),
            ContentType::OctetStream,
            StatusCode::Ok,
            body.to_vec(),
        )
        .expect("Given status 200 and an octet stream, building the response should always succeed")
    }

    #[test]
    fn should_serialize_a_null_response_for_a_day_without_filings() {
        let output = DownloadDailyIndexOutput::without_filings();

        let expected_result = serde_json::json!({ "response": null });

        let result = serde_json::to_value(&output)
            .expect("Given an output without a response, serialization should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_equal_when_only_one_output_has_a_response() {
        let output = DownloadDailyIndexOutput::new(daily_index_response(b"Form Type"));

        let expected_result = false;

        let result = output == DownloadDailyIndexOutput::without_filings();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_display_no_filings_for_a_day_without_filings() {
        let output = DownloadDailyIndexOutput::without_filings();

        let expected_result = "\tNo filings on this day";

        let result = output.to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_clear_the_response_when_update_marks_a_day_without_filings() {
        let mut output = DownloadDailyIndexOutput::new(daily_index_response(b"Form Type"));
        let update = DownloadDailyIndexOutputUpdater::builder()
            .response(None)
            .build();

        let expected_result = &DownloadDailyIndexOutput::without_filings();

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = DownloadDailyIndexOutput::new(daily_index_response(b"Form Type"));
        let update = DownloadDailyIndexOutputUpdater::builder().build();

        let expected_result = &DownloadDailyIndexOutput::new(daily_index_response(b"Form Type"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_download_daily_index_output() {
        implements_value_traits::<DownloadDailyIndexOutput>();
    }
}
