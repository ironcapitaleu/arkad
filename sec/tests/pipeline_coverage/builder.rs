use std::pin::pin;

use futures_util::StreamExt;
use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::prelude::*;
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawCompanyFacts;
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
use uuid::Uuid;

/// The storage repository that stores the raw Company Facts of each run.
pub type RawCompanyFactsWriter = FilesystemRepository<Write, RawDocument<RawCompanyFacts>>;

/// Builds a [`RawCompanyFactsWriter`] rooted in the temporary directory. Building touches no disk.
pub fn raw_company_facts_writer() -> RawCompanyFactsWriter {
    FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<RawCompanyFacts>>()
        .root(std::env::temp_dir().join("arkad-pipeline-coverage"))
        .build()
}

/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;
/// Marker type indicating no [`SecClient`] has been provided yet.
pub struct NoClient;
/// Marker type indicating no storage repository has been provided yet.
pub struct NoRepository;

/// Typestate builder for constructing a [`Pipeline`].
pub struct PipelineBuilder<C, S, R> {
    cik: C,
    sec_client: S,
    repository: R,
}

impl PipelineBuilder<NoCik, NoClient, NoRepository> {
    pub(super) const fn new() -> Self {
        Self {
            cik: NoCik,
            sec_client: NoClient,
            repository: NoRepository,
        }
    }
}

impl<S, R> PipelineBuilder<NoCik, S, R> {
    pub fn cik(self, cik: impl Into<String>) -> PipelineBuilder<String, S, R> {
        PipelineBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
            repository: self.repository,
        }
    }
}

impl<C, R> PipelineBuilder<C, NoClient, R> {
    pub fn sec_client(self, sec_client: SecClient) -> PipelineBuilder<C, SecClient, R> {
        PipelineBuilder {
            cik: self.cik,
            sec_client,
            repository: self.repository,
        }
    }
}

impl<C, S> PipelineBuilder<C, S, NoRepository> {
    /// Sets the storage repository that stores the raw Company Facts.
    pub fn repository(
        self,
        repository: RawCompanyFactsWriter,
    ) -> PipelineBuilder<C, S, RawCompanyFactsWriter> {
        PipelineBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            repository,
        }
    }
}

impl PipelineBuilder<String, SecClient, RawCompanyFactsWriter> {
    pub fn build(self) -> Pipeline {
        Pipeline {
            raw_cik: self.cik,
            sec_client: self.sec_client,
            repository: self.repository,
        }
    }
}

/// A full Extract + Transform pipeline for a single CIK.
pub struct Pipeline {
    raw_cik: String,
    sec_client: SecClient,
    repository: RawCompanyFactsWriter,
}

impl Pipeline {
    pub const fn builder() -> PipelineBuilder<NoCik, NoClient, NoRepository> {
        PipelineBuilder::new()
    }

    pub async fn run(self) -> Result<(), StreamError> {
        let execution_id = Uuid::new_v4();
        let state = ExtractSuperState::<ValidateCikFormat, _>::new(
            self.raw_cik,
            self.sec_client,
            execution_id,
            self.repository,
        );
        let mut stream = pin!(state.into_run_stream());

        while let Some(result) = stream.next().await {
            result?;
        }

        Ok(())
    }
}
