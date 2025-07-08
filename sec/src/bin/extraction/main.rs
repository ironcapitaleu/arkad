use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::prepare_sec_request::{
    PrepareSecRequest, PrepareSecRequestContext, PrepareSecRequestInputData,
};
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::prelude::*;

#[tokio::main]
async fn main() {
    let mut validate_cik_state = ValidateCikFormat::default();

    println!("\n=======================================================");
    println!("Initial Validation state:");
    println!("{:.500}", validate_cik_state.to_string().as_str());

    validate_cik_state
        .compute_output_data_async()
        .await
        .expect("Hardcoded default CIK should alaways have a valid format.");

    println!("\n=======================================================");
    println!("Validation state after verifying CIK:");
    println!("{:.500}", validate_cik_state.to_string().as_str());

    println!("\n=======================================================");
    println!("Initial PrepareSecRequest state:");
    let mut prepare_sec_request = PrepareSecRequest::new(
        PrepareSecRequestInputData::new(
            validate_cik_state
                .get_output_data()
                .expect("Should be valid")
                .validated_cik
                .clone(),
            "TestUserAgent".to_string(),
        ),
        PrepareSecRequestContext::new("Test"),
    );
    println!("{:.500}", prepare_sec_request.to_string().as_str());
    prepare_sec_request
        .compute_output_data_async()
        .await
        .expect("PrepareSecRequest should always succeed with a valid CIK.");
    println!("\n=======================================================");
    println!("After PrepareSecRequest output:");
    println!("{:.500}", prepare_sec_request.to_string().as_str());

    println!("\n=======================================================");
    println!("Initial Extract SuperState:");

    let mut super_state = ExtractSuperState::new("1067983");

    super_state
        .compute_output_data_async()
        .await
        .expect("Hardcoded default CIK should alaways have a valid format.");

    let validated_cik = super_state
        .get_current_state()
        .get_output_data()
        .unwrap()
        .validated_cik
        .value();
    println!("State 1 Output: Validated CIK is {validated_cik}");
}
