use sec::sec_state_machine::ingestion::retrieval::Retrieval;
use state_maschine::prelude::*;

fn main() {
    let mut retrieval_state = Retrieval::default();

    println!("\n=======================================================");
    println!("Initial Retrieval state:");
    println!("{:.500}", retrieval_state.to_string().as_str());

    retrieval_state.compute_output_data();

    println!("\n=======================================================");
    println!("Retrieval state after querying SEC API with CIK:");
    println!("{:.500}...", retrieval_state.to_string().as_str());
}
