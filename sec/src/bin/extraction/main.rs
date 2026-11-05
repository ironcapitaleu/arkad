use std::process::ExitCode;

use chrono::{SecondsFormat, Utc};
use sec::implementations::states::extract::company_facts::ExtractCompanyFactsSuperState;
use sec::implementations::states::extract::execute_sec_request::{
    ExecuteCompanyFactsRequest, ExecuteSecRequestContext, ExecuteSecRequestInput,
};
use sec::implementations::states::extract::prepare_sec_request::PrepareSecRequest;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use storage::implementations::repository::filesystem::IngestionMode;
use uuid::Uuid;

use sec::prelude::*;

use sec_client::{USER_AGENT_ENV_VAR, default_user_agent_warning, sec_client, startup_error_cause};
use storage_root::{create_root, raw_company_facts_writer};

// Each binary is its own crate root, so it cannot import another binary's modules. `#[path]`
// compiles each shared file into this binary as a private module, so all binaries build the
// client and the store the same way.
#[path = "../shared/sec_client.rs"]
mod sec_client;
#[path = "../shared/storage_root.rs"]
mod storage_root;

#[tokio::main]
// Let this be here for testing purposes. Will be removed later.
#[allow(clippy::too_many_lines)]
async fn main() -> ExitCode {
    // Each startup failure is logged as JSON. An error returned from `main` would print a second
    // line in `Debug` form, which is not JSON.
    let sec_client = match sec_client() {
        Ok(sec_client) => sec_client,
        Err(error) => {
            let cause = startup_error_cause(error.as_ref());
            log_line(
                "error",
                "batch_startup_failed",
                &format!(
                    "[FailedSecClientCreation] Failed to build the SEC client, Caused by: {cause}"
                ),
                &serde_json::json!({ "env_var": USER_AGENT_ENV_VAR }),
            );
            return ExitCode::FAILURE;
        }
    };
    if let Some(warning) = default_user_agent_warning(&sec_client) {
        log_line(
            "warn",
            "batch_default_user_agent",
            &warning,
            &serde_json::json!({ "env_var": USER_AGENT_ENV_VAR }),
        );
    }
    let repository = raw_company_facts_writer();
    if let Err(error) = create_root(&repository) {
        log_line(
            "error",
            "batch_startup_failed",
            &format!(
                "[FailedStorageRootCreation] Failed to create the storage root, Reason: '{error}'"
            ),
            &serde_json::json!({ "root": repository.root().display().to_string() }),
        );
        return ExitCode::FAILURE;
    }

    let input =
        sec::implementations::states::extract::validate_cik_format::ValidateCikFormatInput::new(
            "1067983",
        );
    let context =
        sec::implementations::states::extract::validate_cik_format::ValidateCikFormatContext::new(
            "1067983",
        );
    let mut validate_cik_state = ValidateCikFormat::new(input, context);

    println!("\n=======================================================");
    println!("Initial Validation state:");
    println!("{:.500}", validate_cik_state.to_string().as_str());

    validate_cik_state
        .compute_output_data_async()
        .await
        .expect("Hardcoded default CIK should alaways have a valid format");

    println!("\n=======================================================");
    println!("Validation state after verifying CIK:");
    println!("{:.500}", validate_cik_state.to_string().as_str());

    println!("\n=======================================================");
    println!("Initial PrepareSecRequest state:");
    let mut prepare_sec_request = PrepareSecRequest::builder()
        .all_company_facts()
        .cik(
            validate_cik_state
                .output_data()
                .expect("Should be valid")
                .validated_cik
                .clone(),
        )
        .build();
    println!("{:.500}", prepare_sec_request.to_string().as_str());
    prepare_sec_request
        .compute_output_data_async()
        .await
        .expect("PrepareSecRequest should always succeed with a valid CIK and user agent");
    println!("\n=======================================================");
    println!("After PrepareSecRequest output:");
    println!("{:.500}", prepare_sec_request.to_string().as_str());

    println!("\n=======================================================");
    println!("Initial ExecuteSecRequest state:");
    let prepare_output = prepare_sec_request
        .output_data()
        .expect("PrepareSecRequest should have output data")
        .clone();

    let mut execute_sec_request = ExecuteCompanyFactsRequest::new(
        ExecuteSecRequestInput::new(sec_client.clone(), prepare_output.request().clone()),
        ExecuteSecRequestContext::new(
            validate_cik_state
                .output_data()
                .expect("Should be valid")
                .validated_cik
                .clone(),
        ),
    );
    println!("{:.500}", execute_sec_request.to_string().as_str());
    execute_sec_request
        .compute_output_data_async()
        .await
        .expect("ExecuteSecRequest should succeed with valid client and request");
    println!("\n=======================================================");
    println!("After ExecuteSecRequest output:");
    println!("{:.500}", execute_sec_request.to_string().as_str());

    println!("\n=======================================================");
    println!("Initial Extract Company Facts SuperState:");

    let mut super_state = ExtractCompanyFactsSuperState::builder()
        .cik("1067983")
        .sec_client(sec_client)
        .execution_id(Uuid::new_v4())
        .ingestion_mode(IngestionMode::Backfill)
        .repository(repository)
        .build();

    super_state
        .compute_output_data_async()
        .await
        .expect("Hardcoded default CIK should alaways have a valid format");

    let validated_cik = super_state
        .current_state()
        .output_data()
        .unwrap()
        .validated_cik
        .value();
    println!("State 1 Output: Validated CIK is {validated_cik}");

    println!("\n=======================================================");
    println!("Transition Extract from ValidateCikFormat to PrepareSecRequest:");

    let mut super_state = super_state
        .transition_to_next_state_sec()
        .expect("Transition should succeed");

    println!("\n=======================================================");
    println!("Current State Print:");

    println!("{super_state}");

    println!("\n=======================================================");
    println!("State 2 Output:");

    super_state
        .compute_output_data_async()
        .await
        .expect("PrepareSecRequest should succeed with valid CIK and user agent");

    let output = super_state
        .current_state()
        .output_data()
        .expect("PrepareSecRequest should have output data");

    println!("{output}");

    println!("\n=======================================================");
    println!("Transition from PrepareSECRequest to ExecuteSECRequest");

    let mut super_state = super_state
        .transition_to_next_state_sec()
        .expect("Transition should succeed");
    println!("{super_state}");

    println!("\n=======================================================");
    println!("State 3 Output:");

    super_state
        .compute_output_data_async()
        .await
        .expect("ExecuteSecRequest should succeed with valid CIK and user agent");

    let execute_output = super_state
        .current_state()
        .output_data()
        .expect("ExecuteSecRequest should have output data");

    println!("{execute_output}");

    ExitCode::SUCCESS
}

/// Writes one log line to stdout, with the five fields of the JSON logger of the other binaries.
///
/// This binary builds without `tracing`, so it cannot use that logger.
fn log_line(level: &str, event: &str, message: &str, context: &serde_json::Value) {
    let line = serde_json::json!({
        "level": level,
        "timestamp": Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
        "event": event,
        "message": message,
        "context": context,
    });
    println!("{line}");
}
