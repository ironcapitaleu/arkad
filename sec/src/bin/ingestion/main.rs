use sec::prelude::*;
use sec::state_machine::extract::validate_cik_format::ValidateCikFormat;

fn main() {
    let mut validate_cik_state = ValidateCikFormat::default();

    println!("\n=======================================================");
    println!("Initial Validation state:");
    println!("{:.500}", validate_cik_state.to_string().as_str());

    validate_cik_state
        .compute_output_data()
        .expect("Hardcoded default CIK should alaways have a valid format.");

    println!("\n=======================================================");
    println!("Validation state after verifying CIK:");
    println!("{:.500}", validate_cik_state.to_string().as_str());
}
