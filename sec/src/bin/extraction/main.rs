use std::env;
use std::path::PathBuf;

use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::execute_sec_request::{
    ExecuteSecRequest, ExecuteSecRequestContext, ExecuteSecRequestInput,
};
use sec::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInput,
};
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::shared::http_client::implementations::sec_client::SecClient;
use sec::shared::raw_document::RawDocument;
use sec::shared::raw_source::RawCompanyFacts;
use storage::implementations::repository::filesystem::FilesystemRepository;
use uuid::Uuid;

use sec::prelude::*;

/// The environment variable that names the root directory of the document store.
const STORAGE_ROOT_ENV_VAR: &str = "ARKAD_STORAGE_ROOT";

/// The root directory of the document store when `ARKAD_STORAGE_ROOT` is unset.
const DEFAULT_STORAGE_ROOT: &str = "data";

#[tokio::main]
// Let this be here for testing purposes. Will be removed later.
#[allow(clippy::too_many_lines)]
async fn main() {
    let sec_client = SecClient::default();
    let storage_root = env::var_os(STORAGE_ROOT_ENV_VAR)
        .map_or_else(|| PathBuf::from(DEFAULT_STORAGE_ROOT), PathBuf::from);
    let repository = FilesystemRepository::builder()
        .writer()
        .document_type::<RawDocument<RawCompanyFacts>>()
        .root(storage_root)
        .build();

    let input =
        sec::implementations::states::extract::validate_cik_format::ValidateCikFormatInput::new(
            "1067983",
        );
    let context =
        sec::implementations::states::extract::validate_cik_format::ValidateCikFormatContext::new(
            "1067983",
            sec_client.clone(),
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
    println!("Initial Extract SuperState:");

    let mut super_state = ExtractSuperState::<ValidateCikFormat, _>::new(
        "1067983",
        sec_client,
        Uuid::new_v4(),
        repository,
    );

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
}
