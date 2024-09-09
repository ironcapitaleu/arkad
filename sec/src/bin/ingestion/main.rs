use sec::sec_state_machine::ingestion::retrieval::Retrieval;
use state_maschine::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let retrieval_state = Retrieval::default();

    println!("Initial Retrieval state:");
    println!("{retrieval_state}");

    // Call the async function and await the result
    retrieval_state.compute_output_new()?;

    println!("\nRetrieval state after quering SEC API with CIK:");
    println!("{retrieval_state}");

    Ok(())
}
