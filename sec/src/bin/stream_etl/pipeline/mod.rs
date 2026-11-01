mod builder;
pub mod constants;

use std::fmt::{self, Display, Formatter};
use std::pin::pin;
use std::time::Instant;

use futures_util::StreamExt;
use sec::implementations::extraction::Backfill;
use sec::prelude::*;
use sec::shared::http_client::implementations::sec_client::SecClient;

use crate::storage_root::RawCompanyFactsWriter;
use builder::{NoCik, NoClient, NoRepository, PipelineBuilder};

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

/// The extract pipeline for a single CIK. It ends after `StoreCompanyFacts` writes the response to
/// the store.
pub struct Pipeline {
    raw_cik: String,
    sec_client: SecClient,
    repository: RawCompanyFactsWriter,
}

impl Pipeline {
    pub const fn builder() -> PipelineBuilder<NoCik, NoClient, NoRepository> {
        PipelineBuilder::new()
    }

    /// Creates a stream that drives the extract pipeline, with the `execution_id` of `backfill`.
    #[must_use]
    pub fn into_stream(self, backfill: &Backfill) -> StateMachineStream {
        let state = backfill
            .company_facts()
            .cik(self.raw_cik)
            .sec_client(self.sec_client)
            .repository(self.repository)
            .build();
        state.into_run_stream()
    }

    /// Runs the pipeline as part of the run of `backfill`, and logs each event.
    pub async fn run(self, backfill: &Backfill) -> Result<(), StreamError> {
        let cik = self.raw_cik.clone();
        let execution_id = backfill.id();
        let pipeline_start = Instant::now();
        let mut stream = pin!(self.into_stream(backfill));

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
