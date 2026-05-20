use super::Pipeline;

use sec::shared::http_client::implementations::sec_client::SecClient;
/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;
/// Marker type indicating no [`SecClient`] has been provided yet.
pub struct NoClient;

/// Typestate builder for constructing a [`Pipeline`].
///
/// Uses a consuming typestate pattern to enforce at compile time
/// that all required fields are provided before building.
pub struct PipelineBuilder<C, S> {
    cik: C,
    sec_client: S,
}

impl PipelineBuilder<NoCik, NoClient> {
    /// Creates a new builder with no fields set.
    pub(super) const fn new() -> Self {
        Self {
            cik: NoCik,
            sec_client: NoClient,
        }
    }
}

impl<S> PipelineBuilder<NoCik, S> {
    /// Sets the CIK for the pipeline.
    pub fn cik(self, cik: impl Into<String>) -> PipelineBuilder<String, S> {
        PipelineBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
        }
    }
}

impl<C> PipelineBuilder<C, NoClient> {
    /// Sets the shared HTTP client for the pipeline.
    pub fn sec_client(self, sec_client: SecClient) -> PipelineBuilder<C, SecClient> {
        PipelineBuilder {
            cik: self.cik,
            sec_client,
        }
    }
}

impl PipelineBuilder<String, SecClient> {
    /// Builds the [`Pipeline`] from the fully configured builder state.
    pub fn build(self) -> Pipeline {
        Pipeline {
            raw_cik: self.cik,
            sec_client: self.sec_client,
        }
    }
}
