use super::{Pipeline, RawCompanyFactsWriter};

use sec::shared::http_client::implementations::sec_client::SecClient;
/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;
/// Marker type indicating no [`SecClient`] has been provided yet.
pub struct NoClient;
/// Marker type indicating no repository has been provided yet.
pub struct NoRepository;

/// Typestate builder for constructing a [`Pipeline`].
///
/// Uses a consuming typestate pattern to enforce at compile time
/// that all required fields are provided before building.
pub struct PipelineBuilder<C, S, P> {
    cik: C,
    sec_client: S,
    repository: P,
}

impl PipelineBuilder<NoCik, NoClient, NoRepository> {
    /// Creates a new builder with no fields set.
    pub(super) const fn new() -> Self {
        Self {
            cik: NoCik,
            sec_client: NoClient,
            repository: NoRepository,
        }
    }
}

impl<S, P> PipelineBuilder<NoCik, S, P> {
    /// Sets the CIK for the pipeline.
    pub fn cik(self, cik: impl Into<String>) -> PipelineBuilder<String, S, P> {
        PipelineBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
            repository: self.repository,
        }
    }
}

impl<C, P> PipelineBuilder<C, NoClient, P> {
    /// Sets the shared HTTP client for the pipeline.
    pub fn sec_client(self, sec_client: SecClient) -> PipelineBuilder<C, SecClient, P> {
        PipelineBuilder {
            cik: self.cik,
            sec_client,
            repository: self.repository,
        }
    }
}

impl<C, S> PipelineBuilder<C, S, NoRepository> {
    /// Sets the repository that stores the raw Company Facts.
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
    /// Builds the [`Pipeline`] from the fully configured builder state.
    pub fn build(self) -> Pipeline {
        Pipeline {
            raw_cik: self.cik,
            sec_client: self.sec_client,
            repository: self.repository,
        }
    }
}
