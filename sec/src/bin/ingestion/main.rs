use sec::sec_state_machine::extract::validate_cik_format::ValidateCikFormat;
use sec::sec_state_machine::ingestion::retrieval::Retrieval;
use state_maschine::prelude::*;

fn main() {
    let mut retrieval_state = Retrieval::default();
    let mut validate_cik_state = ValidateCikFormat::default();

    println!("\n=======================================================");
    println!("Initial Retrieval + Validation state:");
    //println!("{:.500}", retrieval_state.to_string().as_str());
    println!("{:.500}", validate_cik_state.to_string().as_str());

    retrieval_state.compute_output_data();
    validate_cik_state.compute_output_data();

    println!("\n=======================================================");
    println!("Retrieval state after querying SEC API with CIK:");
    //println!("{:.500}...", retrieval_state.to_string().as_str());
    println!("{:.500}", validate_cik_state.to_string().as_str());
}
