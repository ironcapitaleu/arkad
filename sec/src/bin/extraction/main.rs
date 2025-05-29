use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
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
}
