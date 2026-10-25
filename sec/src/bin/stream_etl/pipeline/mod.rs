mod builder;
pub mod constants;

use std::env;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use std::pin::pin;
use std::time::Instant;

use futures_util::StreamExt;
use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::prelude::*;
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawCompanyFacts;
use storage::implementations::repository::filesystem::{FilesystemRepository, Write};
use uuid::Uuid;

use builder::{NoCik, NoClient, NoRepository, PipelineBuilder};
use constants::{DEFAULT_STORAGE_ROOT, STORAGE_ROOT_ENV_VAR};

/// The repository that stores the raw Company Facts of each run.
pub type RawCompanyFactsWriter = FilesystemRepository<Write, RawDocument<RawCompanyFacts>>;

/// Builds the [`RawCompanyFactsWriter`] for the root that `ARKAD_STORAGE_ROOT` names.
///
/// If the variable is unset, the root is [`DEFAULT_STORAGE_ROOT`]. Building touches no disk.
pub fn raw_company_facts_writer() -> RawCompanyFactsWriter {
    let root = env::var_os(STORAGE_ROOT_ENV_VAR)
        .map_or_else(|| PathBuf::from(DEFAULT_STORAGE_ROOT), PathBuf::from);
    FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<RawCompanyFacts>>()
        .root(root)
        .build()
}

/// Events emitted by the pipeline runner (consumer-level, not part of the library).
enum PipelineEvent {
    Complete,
    Failed,
}

impl Display for PipelineEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete => write!(f, "pipeline_complete"),
            Self::Failed => write!(f, "pipeline_failed"),
        }
    }
}

/// A full Extract + Transform pipeline for a single CIK.
///
/// Creates an `ExtractSuperState` and calls `into_stream()` -- the framework
/// automatically chains through Transform via the cross-`SuperState` transition.
pub struct Pipeline {
    raw_cik: String,
    sec_client: SecClient,
    repository: RawCompanyFactsWriter,
}

impl Pipeline {
    pub const fn builder() -> PipelineBuilder<NoCik, NoClient, NoRepository> {
        PipelineBuilder::new()
    }

    /// Creates a stream that drives the full ETL pipeline (Extract → Transform).
    #[must_use]
    pub fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        let state = ExtractSuperState::<ValidateCikFormat, _>::new(
            self.raw_cik,
            self.sec_client,
            execution_id,
            self.repository,
        );
        state.into_stream(execution_id)
    }

    pub async fn run(self) -> Result<(), StreamError> {
        let cik = self.raw_cik.clone();
        let execution_id = Uuid::new_v4();
        let pipeline_start = Instant::now();
        let mut stream = pin!(self.into_stream(execution_id));

        let mut stream_error: Option<StreamError> = None;
        while let Some(result) = stream.next().await {
            match result {
                Ok(item) => {
                    tracing::info!(
                        event = %item.event,
                        message = %format!("{}: '{}'", item.event, item.state_name),
                        event_duration_ms = u32::try_from(item.event_duration.as_millis()).unwrap_or(u32::MAX),
                        execution_id = %execution_id,
                        cik = %cik,
                        state = %item.state_name,
                        data = %item.data,
                    );
                }
                Err(e) => {
                    tracing::error!(
                        event = %e.event,
                        message = %e.source.to_string(),
                        duration_ms = pipeline_start.elapsed().as_millis(),
                        execution_id = %e.execution_id,
                        cik = %cik,
                        state = %e.state_name,
                        data = %e.data,
                    );
                    stream_error = Some(e);
                    break;
                }
            }
        }

        let pipeline_duration = pipeline_start.elapsed();
        if let Some(e) = stream_error {
            tracing::warn!(
                event = %PipelineEvent::Failed,
                message = %format!("Pipeline for CIK '{cik}' failed after {pipeline_duration:.2?}"),
                execution_id = %execution_id,
                cik = %cik,
                duration_ms = pipeline_duration.as_millis(),
            );
            return Err(e);
        }

        tracing::info!(
            event = %PipelineEvent::Complete,
            message = %format!("Pipeline for CIK '{cik}' completed in {pipeline_duration:.2?}"),
            execution_id = %execution_id,
            cik = %cik,
            duration_ms = pipeline_duration.as_millis(),
        );

        Ok(())
    }
}
