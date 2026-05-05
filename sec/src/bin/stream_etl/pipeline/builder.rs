use super::Pipeline;

/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;

/// Typestate builder for constructing a [`Pipeline`].
///
/// Uses a consuming typestate pattern to enforce at compile time
/// that all required fields are provided before building.
pub struct PipelineBuilder<C> {
    cik: C,
}

impl PipelineBuilder<NoCik> {
    /// Creates a new builder with no fields set.
    pub(super) const fn new() -> Self {
        Self { cik: NoCik }
    }

    /// Sets the CIK for the pipeline.
    #[allow(clippy::unused_self)]
    pub fn cik(self, cik: impl Into<String>) -> PipelineBuilder<String> {
        PipelineBuilder { cik: cik.into() }
    }
}

impl PipelineBuilder<String> {
    /// Builds the [`Pipeline`] from the fully configured builder state.
    pub fn build(self) -> Pipeline {
        Pipeline { raw_cik: self.cik }
    }
}
