use super::Extraction;

/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;

/// Typestate builder for constructing an [`Extraction`].
///
/// Uses a consuming typestate pattern to enforce at compile time
/// that all required fields are provided before building.
pub struct ExtractionBuilder<C> {
    cik: C,
}

impl ExtractionBuilder<NoCik> {
    /// Creates a new builder with no fields set.
    pub(super) const fn new() -> Self {
        Self { cik: NoCik }
    }

    /// Sets the CIK for the extraction.
    #[allow(clippy::unused_self)]
    pub fn cik(self, cik: impl Into<String>) -> ExtractionBuilder<String> {
        ExtractionBuilder { cik: cik.into() }
    }
}

impl ExtractionBuilder<String> {
    /// Builds the [`Extraction`] from the fully configured builder state.
    pub fn build(self) -> Extraction {
        Extraction { raw_cik: self.cik }
    }
}
