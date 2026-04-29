mod builder;
pub mod constants;

use std::fmt::{self, Display, Formatter};
use std::pin::pin;
use std::time::Instant;

use futures_util::StreamExt;
use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::prelude::*;
use uuid::Uuid;

use builder::{ExtractionBuilder, NoCik};

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

pub struct Extraction {
    raw_cik: String,
}

impl Extraction {
    pub const fn builder() -> ExtractionBuilder<NoCik> {
        ExtractionBuilder::new()
    }

    #[must_use]
    pub fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        let state = ExtractSuperState::<ValidateCikFormat>::new(self.raw_cik);
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
                        event_duration_ms = item.event_duration.as_millis(),
                        context = %serde_json::json!({
                            "execution_id": execution_id.to_string(),
                            "cik": cik,
                            "state": item.state_name,
                            "data": item.data,
                        }),
                    );
                }
                Err(e) => {
                    tracing::error!(
                        event = %e.event,
                        message = %e.source.to_string(),
                        duration_ms = pipeline_start.elapsed().as_millis(),
                        context = %serde_json::json!({
                            "execution_id": e.execution_id.to_string(),
                            "cik": cik,
                            "state": e.state_name,
                            "data": e.data,
                        }),
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
                context = %serde_json::json!({
                    "execution_id": execution_id.to_string(),
                    "cik": cik,
                    "duration_ms": pipeline_duration.as_millis(),
                }),
            );
            return Err(e);
        }

        tracing::info!(
            event = %PipelineEvent::Complete,
            message = %format!("Pipeline for CIK '{cik}' completed in {pipeline_duration:.2?}"),
            context = %serde_json::json!({
                "execution_id": execution_id.to_string(),
                "cik": cik,
                "duration_ms": pipeline_duration.as_millis(),
            }),
        );

        Ok(())
    }
}
