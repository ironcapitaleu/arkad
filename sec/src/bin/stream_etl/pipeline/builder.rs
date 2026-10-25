use super::Pipeline;

use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::user_agent::UserAgent;

use crate::storage_root::RawCompanyFactsWriter;
/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;
/// Marker type indicating no [`SecClient`] has been provided yet.
pub struct NoClient;
/// Marker type indicating no storage repository has been provided yet.
pub struct NoRepository;
/// Marker type indicating no [`UserAgent`] has been provided yet.
pub struct NoUserAgent;

/// Typestate builder for constructing a [`Pipeline`].
///
/// Uses a consuming typestate pattern to enforce at compile time
/// that all required fields are provided before building.
pub struct PipelineBuilder<C, S, R, U> {
    cik: C,
    sec_client: S,
    repository: R,
    user_agent: U,
}

impl PipelineBuilder<NoCik, NoClient, NoRepository, NoUserAgent> {
    /// Creates a new builder with no fields set.
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
    /// Sets the CIK for the pipeline.
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
    /// Sets the shared HTTP client for the pipeline.
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
    /// Sets the `User-Agent` of the run. It must equal the header that the [`SecClient`] sends.
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
    /// Builds the [`Pipeline`] from the fully configured builder state.
    pub fn build(self) -> Pipeline {
        Pipeline {
            raw_cik: self.cik,
            sec_client: self.sec_client,
            repository: self.repository,
            user_agent: self.user_agent,
        }
    }
}
