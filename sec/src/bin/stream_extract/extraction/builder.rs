use super::Extraction;

use sec::shared::http_client::implementations::sec_client::SecClient;

use crate::storage_root::RawCompanyFactsWriter;
/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;
/// Marker type indicating no [`SecClient`] has been provided yet.
pub struct NoClient;
/// Marker type indicating no storage repository has been provided yet.
pub struct NoRepository;

/// Typestate builder for constructing an [`Extraction`].
///
/// Uses a consuming typestate pattern to enforce at compile time
/// that all required fields are provided before building.
pub struct ExtractionBuilder<C, S, R> {
    cik: C,
    sec_client: S,
    repository: R,
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

impl<S, R> ExtractionBuilder<NoCik, S, R> {
    /// Sets the CIK for the extraction.
    pub fn cik(self, cik: impl Into<String>) -> ExtractionBuilder<String, S, R> {
        ExtractionBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
            repository: self.repository,
        }
    }
}

impl<C, R> ExtractionBuilder<C, NoClient, R> {
    /// Sets the shared HTTP client for the extraction.
    pub fn sec_client(self, sec_client: SecClient) -> ExtractionBuilder<C, SecClient, R> {
        ExtractionBuilder {
            cik: self.cik,
            sec_client,
            repository: self.repository,
        }
    }
}

impl<C, S> ExtractionBuilder<C, S, NoRepository> {
    /// Sets the storage repository that stores the raw Company Facts.
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
