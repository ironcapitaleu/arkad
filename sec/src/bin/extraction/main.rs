use std::error::Error;

use sec::implementations::states::extract::ExtractCompanyFactsSuperState;
use sec::implementations::states::extract::execute_sec_request::{
    ExecuteSecRequest, ExecuteSecRequestContext, ExecuteSecRequestInput,
};
use sec::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use uuid::Uuid;

use sec::prelude::*;

use sec_client::{default_user_agent_warning, sec_client};
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
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let sec_client = sec_client()?;
    if let Some(warning) = default_user_agent_warning(&sec_client) {
        eprintln!("Warning: {warning}");
    }
    let repository = raw_company_facts_writer();
    create_root(&repository)?;

    let input =
        sec::implementations::states::extract::validate_cik_format::ValidateCikFormatInput::new(
            "1067983",
        );
    let context =
        sec::implementations::states::extract::validate_cik_format::ValidateCikFormatContext::new(
            "1067983",
            sec_client.clone(),
            repository.clone(),
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
    let mut prepare_sec_request = PrepareSecRequest::new(
        PrepareSecRequestInput::new(
            validate_cik_state
                .output_data()
                .expect("Should be valid")
                .validated_cik
                .clone(),
            sec_client.clone(),
        ),
        PrepareSecRequestContext::new(
            validate_cik_state
                .output_data()
                .expect("Should be valid")
                .validated_cik
                .clone(),
            repository.clone(),
        ),
    );
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

    let mut execute_sec_request = ExecuteSecRequest::new(
        ExecuteSecRequestInput::new(
            prepare_output.client().clone(),
            prepare_output.request().clone(),
        ),
        ExecuteSecRequestContext::new(
            validate_cik_state
                .output_data()
                .expect("Should be valid")
                .validated_cik
                .clone(),
            repository.clone(),
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

    Ok(())
}
