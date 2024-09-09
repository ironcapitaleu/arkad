use sec::sec_state_machine::ingestion::retrieval::Retrieval;
use state_maschine::prelude::*;

fn main() {
    let mut retrieval_state = Retrieval::default();

    println!("Initial Retrieval state:");
    println!("{retrieval_state}");

    // Call the async function and await the result
    retrieval_state.compute_output_data();

    println!("\nRetrieval state after quering SEC API with CIK:");
    println!("{retrieval_state}");
}
