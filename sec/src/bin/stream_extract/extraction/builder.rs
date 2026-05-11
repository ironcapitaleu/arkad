use super::Extraction;

use sec::shared::http_client::implementations::sec_client::SecClient;
/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;
/// Marker type indicating no [`SecClient`] has been provided yet.
pub struct NoClient;

/// Typestate builder for constructing an [`Extraction`].
///
/// Uses a consuming typestate pattern to enforce at compile time
/// that all required fields are provided before building.
pub struct ExtractionBuilder<C, S> {
    cik: C,
    sec_client: S,
}

impl ExtractionBuilder<NoCik, NoClient> {
    /// Creates a new builder with no fields set.
    pub(super) const fn new() -> Self {
        Self {
            cik: NoCik,
            sec_client: NoClient,
        }
    }
}

impl<S> ExtractionBuilder<NoCik, S> {
    /// Sets the CIK for the extraction.
    pub fn cik(self, cik: impl Into<String>) -> ExtractionBuilder<String, S> {
        ExtractionBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
        }
    }
}

impl<C> ExtractionBuilder<C, NoClient> {
    /// Sets the shared HTTP client for the extraction.
    pub fn sec_client(self, sec_client: SecClient) -> ExtractionBuilder<C, SecClient> {
        ExtractionBuilder {
            cik: self.cik,
            sec_client,
        }
    }
}

impl ExtractionBuilder<String, SecClient> {
    /// Builds the [`Extraction`] from the fully configured builder state.
    pub fn build(self) -> Extraction {
        Extraction {
            raw_cik: self.cik,
            sec_client: self.sec_client,
        }
    }
}
