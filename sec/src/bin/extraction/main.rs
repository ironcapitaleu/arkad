use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::{
    ValidateCikFormat, ValidateCikFormatContext, ValidateCikFormatInputData,
};
use sec::prelude::*;

use sec::error::{ErrorKind, StateMachine};

#[tokio::main]
async fn main() {
    let err = ErrorKind::StateMachine(StateMachine::InvalidConfiguration);
    println!("Printing error:");
    println!("=======================================================");
    println!("{err}");
    println!("=======================================================");

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
    println!("Initial Extract SuperState:");

    let mut super_state = ExtractSuperState::new(
        ValidateCikFormatInputData::new("1067983"),
        ValidateCikFormatContext::default(),
    );

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
