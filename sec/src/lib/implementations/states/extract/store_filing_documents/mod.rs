//! # Store Filing Documents State
//!
//! Provides the [`StoreFilingDocuments`] state, which writes the downloaded XBRL instance and
//! label file of one filing to the store and outputs a [`FilingDocumentsRef`] to the stored files.
//!
//! ## Modules
//!
//! - [`constants`]: State metadata such as [`STATE_NAME`].
//! - [`context`]: The [`StoreFilingDocumentsContext`] carried alongside the state.
//! - [`data`]: The [`StoreFilingDocumentsInput`] and [`StoreFilingDocumentsOutput`] data types.
//!
//! ## See Also
//!
//! - [`crate::shared::filing_documents_ref`]: The keys of the stored files.

use std::fmt;

use async_trait::async_trait;
use chrono::Utc;
use serde::Serialize;
use state_maschine::prelude::State as SMState;
use storage::WriteError;
use storage::implementations::repository::filesystem::{IngestionMode, InputDocument};

use crate::error::State as StateError;
use crate::error::state_machine::state::FailedDocumentWrite;
use crate::shared::build_info::producer;
use crate::shared::document_storage::FailedDocumentStorage;
use crate::shared::filing_documents_ref::{FilingDocumentsRef, instance_key, labels_key};
use crate::shared::raw_document::{FilingFields, RawDocument};
use crate::shared::raw_repository::RawRepository;
use crate::shared::raw_source::{RawXbrlInstance, RawXbrlLabels};
use crate::shared::run_fields::RunFields;
use crate::traits::state_machine::state::State;

pub mod constants;
pub mod context;
pub mod data;

pub use constants::STATE_NAME;
pub use context::StoreFilingDocumentsContext;
pub use data::StoreFilingDocumentsInput;
pub use data::StoreFilingDocumentsOutput;

/// Writes the downloaded XBRL instance and label file of one filing to the store.
///
/// Builds a [`RawDocument<RawXbrlInstance>`](crate::shared::raw_source::RawXbrlInstance) and, if
/// the filing has a label file, a [`RawDocument<RawXbrlLabels>`](crate::shared::raw_source::RawXbrlLabels)
/// with [`RawDocument::from_filing_file`]. Writes them under the keys of
/// [`instance_key`] and [`labels_key`] through the repositories of type `I` and `L`, and outputs a
/// [`FilingDocumentsRef`]. The metadata records the ingestion mode [`IngestionMode::Incremental`]
/// and the filing as [`FilingFields`].
///
/// The instance is written first. If the label write fails, the instance stays stored and the
/// state has no output. A new run stores the same instance bytes, which the repository skips.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct StoreFilingDocuments<I, L> {
    input: StoreFilingDocumentsInput,
    context: StoreFilingDocumentsContext<I, L>,
    output: Option<StoreFilingDocumentsOutput>,
}

impl<I, L> StoreFilingDocuments<I, L> {
    /// Creates a new [`StoreFilingDocuments`] state with no output computed yet.
    #[must_use]
    pub const fn new(
        input: StoreFilingDocumentsInput,
        context: StoreFilingDocumentsContext<I, L>,
    ) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }

    /// Returns this state with the documents that the filing came from, which the metadata of
    /// the stored files records as inputs.
    #[must_use]
    pub fn with_inputs(self, inputs: Vec<InputDocument>) -> Self {
        Self {
            context: self.context.with_inputs(inputs),
            ..self
        }
    }

    /// Consumes the state and returns its input, optional output, and context.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        StoreFilingDocumentsInput,
        Option<StoreFilingDocumentsOutput>,
        StoreFilingDocumentsContext<I, L>,
    ) {
        (self.input, self.output, self.context)
    }
}

/// Wraps a rejected write of this state.
fn failed_write(error: WriteError) -> StateError {
    FailedDocumentWrite::new(STATE_NAME, FailedDocumentStorage::from(error)).into()
}

#[async_trait]
impl<I, L> State for StoreFilingDocuments<I, L>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
{
    /// Writes the instance, then the label file if the filing has one, and stores a reference to
    /// the stored files as output.
    ///
    /// The input gives up its responses once they pass the checks, even if a write fails.
    ///
    /// # Errors
    ///
    /// - Returns [`StateError::InvalidInput`] if the input holds no responses, or if `labels_url`
    ///   is `Some` and the responses hold no label file, or the other way round. The state then
    ///   writes nothing, and the input keeps its responses.
    /// - Returns [`StateError::FailedDocumentWrite`] if a repository rejects a write.
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let has_label_response = self
            .input
            .responses
            .as_deref()
            .ok_or(StateError::InvalidInput)?
            .labels()
            .is_some();
        if has_label_response != self.input.labels_url.is_some() {
            return Err(StateError::InvalidInput);
        }
        let responses = self
            .input
            .responses
            .take()
            .ok_or(StateError::InvalidInput)?;
        let filing = &self.input.filing;
        let run = RunFields {
            produced_at: Utc::now(),
            execution_id: self.context.execution_id,
            producer: producer(STATE_NAME),
            ingestion_mode: IngestionMode::Incremental,
        };
        let labels =
            responses
                .labels()
                .zip(self.input.labels_url.as_ref())
                .map(|(response, labels_url)| {
                    RawDocument::<RawXbrlLabels>::from_filing_file(
                        labels_key(filing),
                        response,
                        labels_url,
                        &self.input.user_agent,
                        run.clone(),
                        FilingFields::from(filing),
                        self.context.inputs.clone(),
                    )
                });
        let instance = RawDocument::<RawXbrlInstance>::from_filing_file(
            instance_key(filing),
            responses.instance(),
            &self.input.instance_url,
            &self.input.user_agent,
            run,
            FilingFields::from(filing),
            self.context.inputs.clone(),
        );
        let documents = FilingDocumentsRef::new(filing.clone(), labels.is_some());

        self.context
            .instance_repository
            .persist(instance)
            .await
            .map_err(failed_write)?;
        if let Some(labels) = labels {
            self.context
                .labels_repository
                .persist(labels)
                .await
                .map_err(failed_write)?;
        }
        self.output = Some(StoreFilingDocumentsOutput::new(documents));
        Ok(())
    }
}

impl<I, L> SMState for StoreFilingDocuments<I, L>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
{
    type InputData = StoreFilingDocumentsInput;
    type OutputData = StoreFilingDocumentsOutput;
    type Context = StoreFilingDocumentsContext<I, L>;

    fn state_name(&self) -> impl ToString {
        STATE_NAME
    }

    /// Blocking wrapper around [`compute_output_data_async`](crate::traits::state_machine::state::State::compute_output_data_async).
    ///
    /// # Panics
    ///
    /// - Panics if the async computation returns an error.
    /// - Panics if called inside a `current_thread` runtime, because `block_in_place` requires a
    ///   multi-thread runtime.
    fn compute_output_data(&mut self) {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(self.compute_output_data_async()))
        } else {
            tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime for blocking compute")
                .block_on(self.compute_output_data_async())
        };

        if let Err(e) = result {
            panic!("compute_output_data failed: {e}")
        }
    }

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl<I, L> fmt::Display for StoreFilingDocuments<I, L>
where
    I: RawRepository<RawXbrlInstance>,
    L: RawRepository<RawXbrlLabels>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;
    use std::{fmt::Debug, hash::Hash};

    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use storage::implementations::repository::filesystem::{
        Document, DocumentKey, FilesystemRepository, ReadWrite,
    };
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;
    use crate::shared::cik::Cik;
    use crate::shared::content_type::ContentType;
    use crate::shared::document_storage::DocumentStorageErrorReason;
    use crate::shared::filing_documents::FilingDocumentResponses;
    use crate::shared::filing_ref::FilingRef;
    use crate::shared::financial::accession_number::AccessionNumber;
    use crate::shared::financial::form::{Form, ParsedForm};
    use crate::shared::headers::Headers;
    use crate::shared::response::implementations::sec_response::{SecResponse, XmlBody};
    use crate::shared::status_code::StatusCode;
    use crate::shared::url::Url;
    use crate::shared::user_agent::UserAgent;
    use crate::tests::fixtures::sample_write_repository::{
        FakeXbrlInstanceRepository, FakeXbrlLabelsRepository,
    };

    type InstanceFilesystemRepository =
        FilesystemRepository<ReadWrite, RawDocument<RawXbrlInstance>>;
    type LabelsFilesystemRepository = FilesystemRepository<ReadWrite, RawDocument<RawXbrlLabels>>;
    type FakeState = StoreFilingDocuments<FakeXbrlInstanceRepository, FakeXbrlLabelsRepository>;

    const INSTANCE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_htm.xml";
    const LABELS_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/000000354526000047/alco-20260908_lab.xml";
    const REDIRECTED_INSTANCE_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/redirected/alco-20260908_htm.xml";
    const REDIRECTED_LABELS_URL: &str =
        "https://www.sec.gov/Archives/edgar/data/3545/redirected/alco-20260908_lab.xml";
    const INSTANCE_KEY: &str = "sec/filings/CIK0000003545/0000003545-26-000047/instance.xml";
    const LABELS_KEY: &str = "sec/filings/CIK0000003545/0000003545-26-000047/lab.xml";
    const INSTANCE_BODY: &[u8] = b"<xbrl>instance</xbrl>";
    const LABELS_BODY: &[u8] = b"<link:linkbase>labels</link:linkbase>";
    const CONFLICT_REASON: &str = "A document already exists under the key";

    fn url(raw_url: &str) -> Url {
        raw_url
            .parse()
            .expect("Given a hardcoded valid URL, parsing should always succeed")
    }

    fn key(path: &str) -> DocumentKey {
        DocumentKey::new(path)
            .expect("Given a hardcoded valid relative path, the key should always build")
    }

    fn filing() -> FilingRef {
        FilingRef::new(
            Cik::new("3545").expect("Given a hardcoded valid CIK, parsing should always succeed"),
            AccessionNumber::new("0000003545-26-000047").expect(
                "Given a hardcoded accession number of the right shape, creation should always succeed",
            ),
            ParsedForm::new(Form::TenQ, false),
            NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
        )
    }

    fn xml_response(raw_url: &str, body: &[u8]) -> SecResponse<XmlBody> {
        SecResponse::from_parts(
            url(raw_url),
            Headers::new(HashMap::from([(
                "content-type".to_string(),
                "text/xml".to_string(),
            )])),
            ContentType::Xml,
            StatusCode::Ok,
            body.to_vec(),
        )
        .expect("Given status 200 and text/xml, building the response should always succeed")
    }

    fn responses(has_label_file: bool) -> Arc<FilingDocumentResponses> {
        Arc::new(FilingDocumentResponses::new(
            xml_response(INSTANCE_URL, INSTANCE_BODY),
            has_label_file.then(|| xml_response(LABELS_URL, LABELS_BODY)),
        ))
    }

    /// Builds an input with a label file whose responses came from other URLs than the input
    /// URLs, as after a redirect.
    fn redirected_input() -> StoreFilingDocumentsInput {
        let mut input = input(true, true);
        input.responses = Some(Arc::new(FilingDocumentResponses::new(
            xml_response(REDIRECTED_INSTANCE_URL, INSTANCE_BODY),
            Some(xml_response(REDIRECTED_LABELS_URL, LABELS_BODY)),
        )));
        input
    }

    /// Returns the filing fields of [`filing`].
    fn expected_filing_fields() -> FilingFields {
        FilingFields {
            filed_date: NaiveDate::from_ymd_opt(2026, 9, 9)
                .expect("Given a hardcoded calendar date, the date should always build"),
            form: "10-Q".to_string(),
            is_amendment: false,
        }
    }

    /// Builds an input whose responses hold a label file if `has_label_response`, and whose
    /// `labels_url` is set if `has_labels_url`.
    fn input(has_label_response: bool, has_labels_url: bool) -> StoreFilingDocumentsInput {
        StoreFilingDocumentsInput::new(
            responses(has_label_response),
            filing(),
            url(INSTANCE_URL),
            has_labels_url.then(|| url(LABELS_URL)),
            UserAgent::new("arkad contact@example.com").expect(
                "Given a hardcoded SEC-format user agent, validation should always succeed",
            ),
        )
    }

    fn state<I, L>(
        input: StoreFilingDocumentsInput,
        instance_repository: I,
        labels_repository: L,
    ) -> StoreFilingDocuments<I, L> {
        StoreFilingDocuments::new(
            input,
            StoreFilingDocumentsContext::new(Uuid::nil(), instance_repository, labels_repository),
        )
    }

    fn fake_state(has_label_file: bool) -> FakeState {
        state(
            input(has_label_file, has_label_file),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
    }

    fn rejecting_write() -> WriteError {
        WriteError::conflicting_write(CONFLICT_REASON)
    }

    fn temporary_root() -> TempDir {
        TempDir::new().expect(
            "Given a writable system temp directory, creating a directory in it should always succeed",
        )
    }

    fn filesystem_repositories(
        root: &Path,
    ) -> (InstanceFilesystemRepository, LabelsFilesystemRepository) {
        let instances = FilesystemRepository::builder()
            .read_write()
            .document_type::<RawDocument<RawXbrlInstance>>()
            .root(root)
            .build();
        let labels = FilesystemRepository::builder()
            .read_write()
            .document_type::<RawDocument<RawXbrlLabels>>()
            .root(root)
            .build();
        (instances, labels)
    }

    /// Runs a state over two filesystem repositories under `root` and returns them.
    async fn store_on_filesystem(
        root: &Path,
        has_label_file: bool,
    ) -> (InstanceFilesystemRepository, LabelsFilesystemRepository) {
        let (instances, labels) = filesystem_repositories(root);
        state(
            input(has_label_file, has_label_file),
            instances.clone(),
            labels.clone(),
        )
        .compute_output_data_async()
        .await
        .expect("Given a fresh writable temp directory, the store should always succeed");
        (instances, labels)
    }

    /// Runs a state over fake repositories and returns them with the result.
    async fn store_on_fakes(
        input: StoreFilingDocumentsInput,
        instances: FakeXbrlInstanceRepository,
        labels: FakeXbrlLabelsRepository,
    ) -> (
        Result<(), StateError>,
        FakeXbrlInstanceRepository,
        FakeXbrlLabelsRepository,
    ) {
        let mut state = state(input, instances.clone(), labels.clone());
        let result = state.compute_output_data_async().await;
        (result, instances, labels)
    }

    fn read_file(path: &Path) -> Vec<u8> {
        std::fs::read(path).expect(
            "Given a document this test just stored under the key, reading it should always succeed",
        )
    }

    #[test]
    fn should_return_state_name_for_store_filing_documents() {
        let state = fake_state(true);

        let expected_result = "Store Filing Documents";

        let result = state.state_name().to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_when_state_has_not_computed_the_output() {
        let state = fake_state(true);

        let expected_result = false;

        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_both_keys_when_the_filing_has_a_label_file() {
        let root = temporary_root();
        let (instances, labels) = filesystem_repositories(root.path());
        let mut state = state(input(true, true), instances, labels);

        let expected_result = Some((key(INSTANCE_KEY), Some(key(LABELS_KEY))));

        state
            .compute_output_data_async()
            .await
            .expect("Given a fresh writable temp directory, the store should always succeed");
        let result = state.output_data().map(|output| {
            (
                output.documents.instance_key().clone(),
                output.documents.labels_key().cloned(),
            )
        });

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_output_no_labels_key_when_the_filing_has_no_label_file() {
        let root = temporary_root();
        let (instances, labels) = filesystem_repositories(root.path());
        let mut state = state(input(false, false), instances, labels);

        let expected_result = Some((key(INSTANCE_KEY), None));

        state
            .compute_output_data_async()
            .await
            .expect("Given a fresh writable temp directory, the store should always succeed");
        let result = state.output_data().map(|output| {
            (
                output.documents.instance_key().clone(),
                output.documents.labels_key().cloned(),
            )
        });

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_instance_bytes_unchanged_under_the_instance_key() {
        let root = temporary_root();
        let (instances, _labels) = store_on_filesystem(root.path(), true).await;

        let expected_result = INSTANCE_BODY;

        let result = read_file(&instances.document_path(&key(INSTANCE_KEY)));

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_store_the_label_bytes_unchanged_under_the_labels_key() {
        let root = temporary_root();
        let (_instances, labels) = store_on_filesystem(root.path(), true).await;

        let expected_result = LABELS_BODY;

        let result = read_file(&labels.document_path(&key(LABELS_KEY)));

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_label_file_when_the_filing_has_no_label_file() {
        let root = temporary_root();
        let (_instances, labels) = store_on_filesystem(root.path(), false).await;

        let expected_result = false;

        let result = labels.document_path(&key(LABELS_KEY)).exists();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_incremental_as_the_ingestion_mode_for_filing_documents() {
        let (_, instances, _) = store_on_fakes(
            input(true, true),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        let expected_result = Some(IngestionMode::Incremental);

        let result = instances
            .persisted()
            .pop()
            .map(|document| document.metadata().ingestion_mode());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_sent_instance_url_when_the_response_came_from_a_redirect() {
        let (_, instances, _) = store_on_fakes(
            redirected_input(),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        let expected_result = Some(url(INSTANCE_URL));

        let result = instances
            .persisted()
            .pop()
            .map(|document| document.metadata().context().requested_url.clone());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_sent_labels_url_when_the_response_came_from_a_redirect() {
        let (_, _, labels) = store_on_fakes(
            redirected_input(),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        let expected_result = Some(url(LABELS_URL));

        let result = labels
            .persisted()
            .pop()
            .map(|document| document.metadata().context().requested_url.clone());

        assert_eq!(result, expected_result);
    }

    fn sample_inputs() -> Vec<InputDocument> {
        vec![InputDocument::new(
            DocumentKey::new("sec/daily-index/form.20260909.idx")
                .expect("Given a hardcoded valid relative path, the key should always build"),
            storage::Sha256Digest::of(b"index"),
        )]
    }

    #[tokio::test]
    async fn should_record_the_inputs_in_the_instance_metadata_when_the_state_has_inputs() {
        let instances = FakeXbrlInstanceRepository::new();
        let mut state = state(
            input(true, true),
            instances.clone(),
            FakeXbrlLabelsRepository::new(),
        )
        .with_inputs(sample_inputs());

        let expected_result = Some(sample_inputs());

        state.compute_output_data_async().await.expect(
            "Given responses for both files and fake repositories, storing should always succeed",
        );
        let result = instances
            .persisted()
            .pop()
            .map(|document| document.metadata().inputs().to_vec());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_inputs_in_the_labels_metadata_when_the_state_has_inputs() {
        let labels = FakeXbrlLabelsRepository::new();
        let mut state = state(
            input(true, true),
            FakeXbrlInstanceRepository::new(),
            labels.clone(),
        )
        .with_inputs(sample_inputs());

        let expected_result = Some(sample_inputs());

        state.compute_output_data_async().await.expect(
            "Given responses for both files and fake repositories, storing should always succeed",
        );
        let result = labels
            .persisted()
            .pop()
            .map(|document| document.metadata().inputs().to_vec());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_filed_date_and_the_form_for_the_instance_metadata() {
        let (_, instances, _) = store_on_fakes(
            input(true, true),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        let expected_result = Some(expected_filing_fields());

        let result = instances
            .persisted()
            .pop()
            .and_then(|document| document.metadata().context().filing.clone());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_record_the_filed_date_and_the_form_for_the_labels_metadata() {
        let (_, _, labels) = store_on_fakes(
            input(true, true),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        let expected_result = Some(expected_filing_fields());

        let result = labels
            .persisted()
            .pop()
            .and_then(|document| document.metadata().context().filing.clone());

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_input_when_the_input_holds_no_responses() {
        let mut state = fake_state(true);
        state
            .compute_output_data_async()
            .await
            .expect("Given fake repositories that accept every write, the state should succeed");

        let expected_result = Err(StateError::InvalidInput);

        let result = state.compute_output_data_async().await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_input_when_the_labels_url_has_no_label_response() {
        let mismatched_input = input(false, true);

        let expected_result = Err(StateError::InvalidInput);

        let (result, _, _) = store_on_fakes(
            mismatched_input,
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_invalid_input_when_the_label_response_has_no_labels_url() {
        let mismatched_input = input(true, false);

        let expected_result = Err(StateError::InvalidInput);

        let (result, _, _) = store_on_fakes(
            mismatched_input,
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_instance_when_the_label_response_has_no_labels_url() {
        let mismatched_input = input(true, false);
        let (_, instances, _) = store_on_fakes(
            mismatched_input,
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        let expected_result = 0;

        let result = instances.persisted().len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_instance_when_the_labels_url_has_no_label_response() {
        let mismatched_input = input(false, true);
        let (_, instances, _) = store_on_fakes(
            mismatched_input,
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        let expected_result = 0;

        let result = instances.persisted().len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_responses_when_the_labels_url_has_no_label_response() {
        let mut state = state(
            input(false, true),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::new(),
        );

        let expected_result = true;

        let _ = state.compute_output_data_async().await;
        let result = state.input_data().responses.is_some();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_document_write_when_the_instance_repository_rejects_the_write() {
        let expected_result = Err(StateError::FailedDocumentWrite(FailedDocumentWrite::new(
            STATE_NAME,
            FailedDocumentStorage::new(DocumentStorageErrorReason::ConflictingWrite {
                details: CONFLICT_REASON.to_string(),
            }),
        )));

        let (result, _, _) = store_on_fakes(
            input(true, true),
            FakeXbrlInstanceRepository::rejecting(rejecting_write()),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_return_failed_document_write_when_the_labels_repository_rejects_the_write() {
        let expected_result = Err(StateError::FailedDocumentWrite(FailedDocumentWrite::new(
            STATE_NAME,
            FailedDocumentStorage::new(DocumentStorageErrorReason::ConflictingWrite {
                details: CONFLICT_REASON.to_string(),
            }),
        )));

        let (result, _, _) = store_on_fakes(
            input(true, true),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::rejecting(rejecting_write()),
        )
        .await;

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_write_no_label_file_when_the_instance_repository_rejects_the_write() {
        let (_, _, labels) = store_on_fakes(
            input(true, true),
            FakeXbrlInstanceRepository::rejecting(rejecting_write()),
            FakeXbrlLabelsRepository::new(),
        )
        .await;

        let expected_result = 0;

        let result = labels.persisted().len();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_keep_the_stored_instance_when_the_labels_repository_rejects_the_write() {
        let (_, instances, _) = store_on_fakes(
            input(true, true),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::rejecting(rejecting_write()),
        )
        .await;

        let expected_result = vec![key(INSTANCE_KEY)];

        let result: Vec<DocumentKey> = instances
            .persisted()
            .iter()
            .map(|document| document.key().clone())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[tokio::test]
    async fn should_have_no_output_data_when_the_labels_repository_rejects_the_write() {
        let mut state = state(
            input(true, true),
            FakeXbrlInstanceRepository::new(),
            FakeXbrlLabelsRepository::rejecting(rejecting_write()),
        );

        let expected_result = false;

        let _ = state.compute_output_data_async().await;
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn should_produce_output_when_calling_sync_compute_inside_tokio_runtime() {
        let mut state = fake_state(true);

        let expected_result = true;

        state.compute_output_data();
        let result = state.has_output_data_been_computed();

        assert_eq!(result, expected_result);
    }

    const fn implements_value_traits<
        T: Sized + Send + Sync + Unpin + Clone + Debug + Hash + Eq + Ord,
    >() {
    }
    #[test]
    const fn should_implement_value_traits_for_store_filing_documents() {
        implements_value_traits::<FakeState>();
    }

    const fn implements_state<T: State>() {}
    #[test]
    const fn should_implement_state_for_store_filing_documents_of_filesystem_repositories() {
        implements_state::<
            StoreFilingDocuments<InstanceFilesystemRepository, LabelsFilesystemRepository>,
        >();
    }
}
