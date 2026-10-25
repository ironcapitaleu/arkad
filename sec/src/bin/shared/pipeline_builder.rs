//! # Pipeline Builder
//!
//! Provides the [`Pipeline`] that runs the Extract → Transform phases for one CIK, and the
//! [`PipelineBuilder`] that builds it.
//!
//! Each setter changes the builder's type. [`build`](PipelineBuilder::build) exists only once the
//! builder holds a CIK, an [`SecClient`], and a repository, so a pipeline without one of them does
//! not compile.

use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::builder::{NoCik, NoRepository, NoSecClient};
use sec::implementations::states::extract::store_company_facts::RawCompanyFactsRepository;
use sec::prelude::*;
use sec::shared::http_client::implementations::sec_client::SecClient;
use uuid::Uuid;

/// Runs the Extract → Transform phases for one CIK and stores the raw Company Facts in `R`.
pub struct Pipeline<R> {
    cik: String,
    sec_client: SecClient,
    repository: R,
}

impl<R: RawCompanyFactsRepository + 'static> Pipeline<R> {
    /// Returns a [`PipelineBuilder`] that holds no field yet.
    pub const fn builder() -> PipelineBuilder<NoCik, NoSecClient, NoRepository<R>> {
        PipelineBuilder {
            cik: NoCik,
            sec_client: NoSecClient,
            repository: NoRepository::new(),
        }
    }

    /// Returns the raw CIK of this pipeline.
    // Only the stream binaries log the CIK. `pipeline_coverage` includes this file without calling
    // it, so `dead_code` would fire there.
    #[allow(dead_code)]
    pub fn cik(&self) -> &str {
        &self.cik
    }

    /// Returns the stream that drives the pipeline, tagged with `execution_id`.
    ///
    /// The stream crosses from the extract phase into the transform phase on its own.
    pub fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        ExtractSuperState::builder()
            .cik(self.cik)
            .sec_client(self.sec_client)
            .execution_id(execution_id)
            .repository(self.repository)
            .build()
            .into_run_stream()
    }
}

/// Builds a [`Pipeline`] with a CIK, an [`SecClient`], and a repository.
///
/// Set them with [`cik`](Self::cik), [`sec_client`](Self::sec_client), and
/// [`repository`](Self::repository), in any order. Then call [`build`](Self::build).
pub struct PipelineBuilder<C, S, R> {
    cik: C,
    sec_client: S,
    repository: R,
}

impl<S, R> PipelineBuilder<NoCik, S, R> {
    /// Sets the raw CIK. The extract phase validates it.
    pub fn cik(self, cik: impl Into<String>) -> PipelineBuilder<String, S, R> {
        PipelineBuilder {
            cik: cik.into(),
            sec_client: self.sec_client,
            repository: self.repository,
        }
    }
}

impl<C, R> PipelineBuilder<C, NoSecClient, R> {
    /// Sets the [`SecClient`] that sends the SEC request.
    pub fn sec_client(self, sec_client: SecClient) -> PipelineBuilder<C, SecClient, R> {
        PipelineBuilder {
            cik: self.cik,
            sec_client,
            repository: self.repository,
        }
    }
}

impl<C, S, R> PipelineBuilder<C, S, NoRepository<R>> {
    /// Sets the repository that stores the raw Company Facts.
    pub fn repository(self, repository: R) -> PipelineBuilder<C, S, R> {
        PipelineBuilder {
            cik: self.cik,
            sec_client: self.sec_client,
            repository,
        }
    }
}

impl<R: RawCompanyFactsRepository> PipelineBuilder<String, SecClient, R> {
    /// Builds the [`Pipeline`] with the fields this builder holds.
    pub fn build(self) -> Pipeline<R> {
        Pipeline {
            cik: self.cik,
            sec_client: self.sec_client,
            repository: self.repository,
        }
    }
}
