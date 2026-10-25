use std::pin::pin;

use futures_util::StreamExt;
use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::prelude::*;
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawCompanyFacts;
use sec::shared::user_agent::UserAgent;
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
/// Marker type indicating no [`UserAgent`] has been provided yet.
pub struct NoUserAgent;

/// Typestate builder for constructing a [`Pipeline`].
pub struct PipelineBuilder<C, S, R, U> {
    cik: C,
    sec_client: S,
    repository: R,
    user_agent: U,
}

impl PipelineBuilder<NoCik, NoClient, NoRepository, NoUserAgent> {
    pub(super) const fn new() -> Self {
        Self {
            cik: NoCik,
            sec_client: NoClient,
            repository: NoRepository,
            user_agent: NoUserAgent,
        }
    }
}

impl<S, R, U> PipelineBuilder<NoCik, S, R, U> {
    pub fn cik(self, cik: impl Into<String>) -> PipelineBuilder<String, S, R, U> {
        PipelineBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
            repository: self.repository,
            user_agent: self.user_agent,
        }
    }
}

impl<C, R, U> PipelineBuilder<C, NoClient, R, U> {
    pub fn sec_client(self, sec_client: SecClient) -> PipelineBuilder<C, SecClient, R, U> {
        PipelineBuilder {
            cik: self.cik,
            sec_client,
            repository: self.repository,
            user_agent: self.user_agent,
        }
    }
}

impl<C, S, U> PipelineBuilder<C, S, NoRepository, U> {
    /// Sets the storage repository that stores the raw Company Facts.
    pub fn repository(
        self,
        repository: RawCompanyFactsWriter,
    ) -> PipelineBuilder<C, S, RawCompanyFactsWriter, U> {
        PipelineBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            repository,
            user_agent: self.user_agent,
        }
    }
}

impl<C, S, R> PipelineBuilder<C, S, R, NoUserAgent> {
    /// Sets the `User-Agent` that the run sends to the SEC.
    pub fn user_agent(self, user_agent: UserAgent) -> PipelineBuilder<C, S, R, UserAgent> {
        PipelineBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            repository: self.repository,
            user_agent,
        }
    }
}

impl PipelineBuilder<String, SecClient, RawCompanyFactsWriter, UserAgent> {
    pub fn build(self) -> Pipeline {
        Pipeline {
            raw_cik: self.cik,
            sec_client: self.sec_client,
            repository: self.repository,
            user_agent: self.user_agent,
        }
    }
}

/// A full Extract + Transform pipeline for a single CIK.
pub struct Pipeline {
    raw_cik: String,
    sec_client: SecClient,
    repository: RawCompanyFactsWriter,
    user_agent: UserAgent,
}

impl Pipeline {
    pub const fn builder() -> PipelineBuilder<NoCik, NoClient, NoRepository, NoUserAgent> {
        PipelineBuilder::new()
    }

    pub async fn run(self) -> Result<(), StreamError> {
        let execution_id = Uuid::new_v4();
        let state = ExtractSuperState::<ValidateCikFormat<_>, _>::new(
            self.raw_cik,
            self.sec_client,
            execution_id,
            self.user_agent,
            self.repository,
        );
        let mut stream = pin!(state.into_run_stream());

        while let Some(result) = stream.next().await {
            result?;
        }

        Ok(())
    }
}
