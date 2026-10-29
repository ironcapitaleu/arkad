//! # Download Notes Archive Output
//!
//! Provides the [`DownloadNotesArchiveOutput`] produced by the
//! [`DownloadNotesArchive`](crate::implementations::states::extract::download_notes_archive::DownloadNotesArchive)
//! state, together with its updater and builder.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use state_maschine::prelude::StateData as SMStateData;

use crate::error::State as StateError;
use crate::shared::response::implementations::sec_notes_file_response::SecNotesFileResponse;
use crate::traits::state_machine::state::StateData;

/// Output data of the [`DownloadNotesArchive`](super::super::DownloadNotesArchive) state.
///
/// The response sits behind an [`Arc`], because a `Clone` of the body would copy up to 0.85 GB.
/// A clone of the output copies only the pointer. Equality, ordering, and hashing use only the
/// SHA-256 digest of the body.
#[derive(Debug, Clone)]
pub struct DownloadNotesArchiveOutput {
    /// The downloaded Notes file.
    pub response: Arc<SecNotesFileResponse>,
}

impl DownloadNotesArchiveOutput {
    /// Creates a new [`DownloadNotesArchiveOutput`] from a downloaded Notes file.
    #[must_use]
    pub fn new(response: SecNotesFileResponse) -> Self {
        Self {
            response: Arc::new(response),
        }
    }
}

impl PartialEq for DownloadNotesArchiveOutput {
    fn eq(&self, other: &Self) -> bool {
        self.response.body_digest() == other.response.body_digest()
    }
}

impl Eq for DownloadNotesArchiveOutput {}

impl PartialOrd for DownloadNotesArchiveOutput {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DownloadNotesArchiveOutput {
    fn cmp(&self, other: &Self) -> Ordering {
        self.response
            .body_digest()
            .cmp(other.response.body_digest())
    }
}

impl Hash for DownloadNotesArchiveOutput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.response.body_digest().hash(state);
    }
}

impl StateData for DownloadNotesArchiveOutput {
    /// Replaces the response if the updater sets it.
    fn update_state(&mut self, updates: Self::UpdateType) -> Result<(), StateError> {
        if let Some(response) = updates.response {
            self.response = response;
        }
        Ok(())
    }
}

impl SMStateData for DownloadNotesArchiveOutput {
    type UpdateType = DownloadNotesArchiveOutputUpdater;

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

impl fmt::Display for DownloadNotesArchiveOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\tURL: {}, Body SHA-256: {}",
            self.response.url(),
            self.response.body_digest()
        )
    }
}

/// Updater for [`DownloadNotesArchiveOutput`].
///
/// Fields set to `None` stay unchanged when the updater is applied.
#[derive(Debug, Clone)]
pub struct DownloadNotesArchiveOutputUpdater {
    /// The new response.
    pub response: Option<Arc<SecNotesFileResponse>>,
}

impl DownloadNotesArchiveOutputUpdater {
    /// Returns a [`DownloadNotesArchiveOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn builder() -> DownloadNotesArchiveOutputUpdaterBuilder {
        DownloadNotesArchiveOutputUpdaterBuilder::new()
    }
}

/// Fluent builder for a [`DownloadNotesArchiveOutputUpdater`].
#[derive(Debug, Default)]
pub struct DownloadNotesArchiveOutputUpdaterBuilder {
    response: Option<Arc<SecNotesFileResponse>>,
}

impl DownloadNotesArchiveOutputUpdaterBuilder {
    /// Creates a [`DownloadNotesArchiveOutputUpdaterBuilder`] with no field set.
    #[must_use]
    pub const fn new() -> Self {
        Self { response: None }
    }

    /// Sets the response.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn response(mut self, response: Arc<SecNotesFileResponse>) -> Self {
        self.response = Some(response);
        self
    }

    /// Builds the [`DownloadNotesArchiveOutputUpdater`].
    #[must_use]
    pub fn build(self) -> DownloadNotesArchiveOutputUpdater {
        DownloadNotesArchiveOutputUpdater {
            response: self.response,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::hash_map::DefaultHasher;
    use std::fmt::Debug;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::shared::headers::Headers;
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;

    fn notes_file_response(body: &[u8]) -> SecNotesFileResponse {
        let url: Url = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/2024q1_notes.zip"
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed");
        let headers = Headers::new(HashMap::from([(
            "content-type".to_string(),
            "application/zip".to_string(),
        )]));
        SecNotesFileResponse::from_parts(url, headers, StatusCode::Ok, body.to_vec()).expect(
            "Given status 200 and a ZIP content type, building the response should always succeed",
        )
    }

    fn hash_of(output: &DownloadNotesArchiveOutput) -> u64 {
        let mut hasher = DefaultHasher::new();
        output.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn should_share_the_response_allocation_when_the_output_is_cloned() {
        let output = DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));

        let expected_result = true;

        let clone = output.clone();
        let result = Arc::ptr_eq(&output.response, &clone.response);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_be_equal_when_two_outputs_have_the_same_body() {
        let output = DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));
        let other = DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));

        let expected_result = true;

        let result = output == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_be_equal_when_two_outputs_have_different_bodies() {
        let output = DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));
        let other = DownloadNotesArchiveOutput::new(notes_file_response(b"PK!"));

        let expected_result = false;

        let result = output == other;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_order_outputs_by_body_digest() {
        let output = DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));
        let other = DownloadNotesArchiveOutput::new(notes_file_response(b"PK!"));

        let expected_result = output
            .response
            .body_digest()
            .cmp(other.response.body_digest());

        let result = output.cmp(&other);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_hash_equal_outputs_to_the_same_value() {
        let output = DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));
        let other = DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));

        let expected_result = hash_of(&output);

        let result = hash_of(&other);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_update_response_when_update_contains_response() {
        let mut output = DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));
        let update = DownloadNotesArchiveOutputUpdater::builder()
            .response(Arc::new(notes_file_response(b"PK!")))
            .build();

        let expected_result = &DownloadNotesArchiveOutput::new(notes_file_response(b"PK!"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_leave_output_unchanged_when_update_is_empty() {
        let mut output = DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));
        let update = DownloadNotesArchiveOutputUpdater::builder().build();

        let expected_result = &DownloadNotesArchiveOutput::new(notes_file_response(b"PK"));

        SMStateData::update_state(&mut output, update);
        let result = output.state();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_download_notes_archive_output() {
        implements_value_traits::<DownloadNotesArchiveOutput>();
    }
}
