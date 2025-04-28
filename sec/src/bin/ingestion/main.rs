use sec::sec_state_machine::extract::validate_cik_format::ValidateCikFormat;
use state_maschine::prelude::*;

fn main() {
    let mut validate_cik_state = ValidateCikFormat::default();

    println!("\n=======================================================");
    println!("Initial Validation state:");
    println!("{:.500}", validate_cik_state.to_string().as_str());

    validate_cik_state.compute_output_data();

    println!("\n=======================================================");
    println!("Validation state after verifying CIK:");
    println!("{:.500}", validate_cik_state.to_string().as_str());
}
