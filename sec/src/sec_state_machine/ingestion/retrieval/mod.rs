use state_maschine::prelude::*;

pub mod retrieval_context;
pub mod retrieval_data;

pub use retrieval_context::RetrievalContext;
pub use retrieval_data::RetievalData;

pub struct Retrieval {
    input: RetievalData,
    output: Option<RetievalData>,
    context: RetrievalContext,
}
