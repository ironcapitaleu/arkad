use super::{Extraction, RawCompanyFactsWriter};

use sec::shared::http_client::implementations::sec_client::SecClient;
/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;
/// Marker type indicating no [`SecClient`] has been provided yet.
pub struct NoClient;
/// Marker type indicating no repository has been provided yet.
pub struct NoRepository;

/// Typestate builder for constructing an [`Extraction`].
///
/// Uses a consuming typestate pattern to enforce at compile time
/// that all required fields are provided before building.
pub struct ExtractionBuilder<C, S, P> {
    cik: C,
    sec_client: S,
    repository: P,
}

impl ExtractionBuilder<NoCik, NoClient, NoRepository> {
    /// Creates a new builder with no fields set.
    pub(super) const fn new() -> Self {
        Self {
            cik: NoCik,
            sec_client: NoClient,
            repository: NoRepository,
        }
    }
}

impl<S, P> ExtractionBuilder<NoCik, S, P> {
    /// Sets the CIK for the extraction.
    pub fn cik(self, cik: impl Into<String>) -> ExtractionBuilder<String, S, P> {
        ExtractionBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
            repository: self.repository,
        }
    }
}

impl<C, P> ExtractionBuilder<C, NoClient, P> {
    /// Sets the shared HTTP client for the extraction.
    pub fn sec_client(self, sec_client: SecClient) -> ExtractionBuilder<C, SecClient, P> {
        ExtractionBuilder {
            cik: self.cik,
            sec_client,
            repository: self.repository,
        }
    }
}

impl<C, S> ExtractionBuilder<C, S, NoRepository> {
    /// Sets the repository that stores the raw Company Facts.
    pub fn repository(
        self,
        repository: RawCompanyFactsWriter,
    ) -> ExtractionBuilder<C, S, RawCompanyFactsWriter> {
        ExtractionBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            repository,
        }
    }
}

impl ExtractionBuilder<String, SecClient, RawCompanyFactsWriter> {
    /// Builds the [`Extraction`] from the fully configured builder state.
    pub fn build(self) -> Extraction {
        Extraction {
            raw_cik: self.cik,
            sec_client: self.sec_client,
            repository: self.repository,
        }
    }
}
