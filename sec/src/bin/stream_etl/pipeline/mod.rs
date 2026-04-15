mod builder;
pub mod constants;

use std::fmt;

use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::implementations::states::transform::TransformSuperState;
use sec::implementations::states::transform::parse_company_facts::ParseCompanyFacts;
use sec::prelude::*;
use sec::shared::cik::Cik;
use sec::shared::response::SecResponse as SecResponseTrait;
use sec::shared::response::implementations::sec_response::SecResponse;
use uuid::Uuid;

use builder::{NoCik, PipelineBuilder};

/// Events emitted by the ETL pipeline runner.
enum PipelineEvent {
    PhaseStarted,
    StateCompleted,
    TransitionCompleted,
    PhaseCompleted,
    PipelineComplete,
}

impl fmt::Display for PipelineEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PhaseStarted => write!(f, "phase_started"),
            Self::StateCompleted => write!(f, "state_completed"),
            Self::TransitionCompleted => write!(f, "transition_completed"),
            Self::PhaseCompleted => write!(f, "phase_completed"),
            Self::PipelineComplete => write!(f, "pipeline_complete"),
        }
    }
}

type PipelineResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// An Extract + Transform pipeline for a single CIK.
///
/// Chains the Extract `SuperState` into the Transform `SuperState`
/// with structured JSON logging at every step.
pub struct Pipeline {
    raw_cik: String,
}

impl Pipeline {
    pub const fn builder() -> PipelineBuilder<NoCik> {
        PipelineBuilder::new()
    }

    pub async fn run(self) -> PipelineResult<()> {
        let cik = self.raw_cik.clone();
        let execution_id = Uuid::new_v4();
        let pipeline_start = std::time::Instant::now();

        tracing::info!(
            event = "pipeline_started",
            message = %format!("Starting ETL pipeline for CIK '{cik}'"),
            context = %serde_json::json!({ "execution_id": execution_id.to_string(), "cik": cik }),
        );

        let (response, validated_cik, extract_duration) =
            run_extract_phase(&cik, execution_id).await?;
        let transform_duration =
            run_transform_phase(&cik, execution_id, &response, validated_cik).await?;

        let pipeline_duration = pipeline_start.elapsed();
        tracing::info!(
            event = %PipelineEvent::PipelineComplete,
            message = %format!("ETL pipeline for CIK '{cik}' completed in {pipeline_duration:.2?}"),
            context = %serde_json::json!({
                "execution_id": execution_id.to_string(),
                "cik": cik,
                "extract_duration_ms": extract_duration.as_millis(),
                "transform_duration_ms": transform_duration.as_millis(),
                "total_duration_ms": pipeline_duration.as_millis(),
            }),
        );

        Ok(())
    }
}

/// Runs the Extract phase and returns the response body for Transform.
async fn run_extract_phase(
    cik: &str,
    execution_id: Uuid,
) -> PipelineResult<(SecResponse, Cik, std::time::Duration)> {
    let phase_start = std::time::Instant::now();
    log_phase_started(cik, execution_id, "extract");

    // ValidateCikFormat
    let mut sm = ExtractSuperState::<ValidateCikFormat>::new(cik);
    run_state(&mut sm, cik, execution_id, "extract", "Validate CIK Format").await?;

    // → PrepareSecRequest
    let mut sm = sm.transition_to_next_state_sec().inspect_err(|e| {
        log_error(
            cik,
            execution_id,
            "extract",
            &format!("Transition failed: {e}"),
        );
    })?;
    log_transition(
        cik,
        execution_id,
        "extract",
        "Validate CIK Format",
        "Prepare SEC Request",
    );

    // PrepareSecRequest
    run_state(&mut sm, cik, execution_id, "extract", "Prepare SEC Request").await?;

    // → ExecuteSecRequest
    let mut sm = sm.transition_to_next_state_sec().inspect_err(|e| {
        log_error(
            cik,
            execution_id,
            "extract",
            &format!("Transition failed: {e}"),
        );
    })?;
    log_transition(
        cik,
        execution_id,
        "extract",
        "Prepare SEC Request",
        "Execute SEC Request",
    );

    // ExecuteSecRequest
    run_state(&mut sm, cik, execution_id, "extract", "Execute SEC Request").await?;

    // Clone the SecResponse to pass it to the Transform phase.
    // The SuperState doesn't expose a consuming accessor for the inner state,
    // so we clone the output here. The BodyDigest inside is Copy (u64).
    let sec_response = sm
        .current_state()
        .output_data()
        .expect("ExecuteSecRequest should have output after successful computation")
        .response()
        .clone();

    tracing::info!(
        event = "bridge",
        message = %format!("[Bridge] Extract → Transform for CIK '{cik}' ({sec_response})"),
        context = %serde_json::json!({
            "execution_id": execution_id.to_string(),
            "cik": cik,
            "response_url": sec_response.url().to_string(),
            "response_status": sec_response.status_code().to_string(),
        }),
    );

    let validated_cik = Cik::new(cik)?;
    let phase_duration = phase_start.elapsed();
    log_phase_completed(cik, execution_id, "extract", phase_duration);

    Ok((sec_response, validated_cik, phase_duration))
}

/// Runs the Transform phase with the response data from Extract.
async fn run_transform_phase(
    cik: &str,
    execution_id: Uuid,
    response: &SecResponse,
    validated_cik: Cik,
) -> PipelineResult<std::time::Duration> {
    let phase_start = std::time::Instant::now();
    log_phase_started(cik, execution_id, "transform");

    // ParseCompanyFacts
    let mut sm = TransformSuperState::<ParseCompanyFacts>::new(response, validated_cik);
    run_state(
        &mut sm,
        cik,
        execution_id,
        "transform",
        "Parse Company Facts",
    )
    .await?;

    // → CreateFinancialStatements
    let mut sm = sm.transition_to_next_state_sec().inspect_err(|e| {
        log_error(
            cik,
            execution_id,
            "transform",
            &format!("Transition failed: {e}"),
        );
    })?;
    log_transition(
        cik,
        execution_id,
        "transform",
        "Parse Company Facts",
        "Create Financial Statements",
    );

    // CreateFinancialStatements (scaffold)
    run_state(
        &mut sm,
        cik,
        execution_id,
        "transform",
        "Create Financial Statements",
    )
    .await?;

    // Summary
    let facts = sm.current_state().input_data();
    tracing::info!(
        event = "transform_summary",
        message = %format!("[Transform] Parsed {} company facts for '{}'",
            facts.company_data().facts().len(),
            facts.company_data().entity_name()),
        context = %serde_json::json!({
            "execution_id": execution_id.to_string(),
            "cik": cik,
            "facts_count": facts.company_data().facts().len(),
            "entity_name": facts.company_data().entity_name().value(),
        }),
    );

    let phase_duration = phase_start.elapsed();
    log_phase_completed(cik, execution_id, "transform", phase_duration);

    Ok(phase_duration)
}

/// Runs `compute_output_data_async` on the current state and logs the result.
async fn run_state<S: State>(
    sm: &mut impl StateMachine<S>,
    cik: &str,
    execution_id: Uuid,
    phase: &str,
    state_name: &str,
) -> PipelineResult<()> {
    let start = std::time::Instant::now();
    sm.current_state_mut()
        .compute_output_data_async()
        .await
        .map_err(std::convert::Into::into)
        .map_err(|e: sec::error::State| {
            log_error(
                cik,
                execution_id,
                phase,
                &format!("State '{state_name}' failed: {e}"),
            );
            e
        })?;

    tracing::info!(
        event = %PipelineEvent::StateCompleted,
        message = %format!("[{phase}] State '{state_name}' completed"),
        // u32 holds up to ~49 days in milliseconds; no single state will run that long - if it does, we deseve the bug
        event_duration_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX),
        context = %serde_json::json!({
            "execution_id": execution_id.to_string(),
            "cik": cik,
            "phase": phase,
            "state": state_name,
            "duration_ms": start.elapsed().as_millis(),
        }),
    );

    Ok(())
}

fn log_phase_started(cik: &str, execution_id: Uuid, phase: &str) {
    tracing::info!(
        event = %PipelineEvent::PhaseStarted,
        message = %format!("[{phase}] Starting {phase} for CIK '{cik}'"),
        context = %serde_json::json!({
            "execution_id": execution_id.to_string(), "cik": cik, "phase": phase,
        }),
    );
}

fn log_phase_completed(cik: &str, execution_id: Uuid, phase: &str, duration: std::time::Duration) {
    tracing::info!(
        event = %PipelineEvent::PhaseCompleted,
        message = %format!("[{phase}] Completed in {duration:.2?}"),
        context = %serde_json::json!({
            "execution_id": execution_id.to_string(), "cik": cik, "phase": phase,
            "duration_ms": duration.as_millis(),
        }),
    );
}

fn log_transition(cik: &str, execution_id: Uuid, phase: &str, from: &str, to: &str) {
    tracing::info!(
        event = %PipelineEvent::TransitionCompleted,
        message = %format!("[{phase}] Transition: '{from}' → '{to}'"),
        context = %serde_json::json!({
            "execution_id": execution_id.to_string(), "cik": cik, "phase": phase,
            "from_state": from, "to_state": to,
        }),
    );
}

fn log_error(cik: &str, execution_id: Uuid, phase: &str, error: &str) {
    tracing::error!(
        event = "pipeline_failed",
        message = %format!("[{phase}] {error}"),
        context = %serde_json::json!({
            "execution_id": execution_id.to_string(), "cik": cik, "phase": phase, "error": error,
        }),
    );
}
