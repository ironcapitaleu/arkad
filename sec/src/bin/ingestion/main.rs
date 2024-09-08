use sec::sec_state_machine::ingestion::retrieval::Retrieval;
use state_maschine::prelude::*;
use std::error::Error;
// fn main() {
//     let retrieval_state = Retrieval::default();

//     let _state_data = retrieval_state.get_input_data();
//     let context = retrieval_state.get_context_data();

//     println!("{retrieval_state}");

//     println!("CIK to retrieve: {}", context.cik());
//     let retrieval_result = retrieval_state.compute_output_new()
//     .await?;

// }

// Make the main function async using the #[tokio::main] macro
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let retrieval_state = Retrieval::default();

    let _state_data = retrieval_state.get_input_data();
    let context = retrieval_state.get_context_data();

    println!("{retrieval_state}");

    println!("CIK to retrieve: {}", context.cik());

    // Call the async function and await the result
    retrieval_state.compute_output_new().await?;

    Ok(())
}
