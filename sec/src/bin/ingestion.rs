use sec::sec_state_machine::ingestion::retrieval::Retrieval;
use state_maschine::prelude::*;

fn main() {
    let retrieval_state = Retrieval::default();

    // let state_data = retrieval_state.get_input_data();
    // let context = retrieval_state.get_context_data();

    // println!("Context:\n{context}");
    // println!("State data:\n{state_data}");
    println!("{retrieval_state}");
}
