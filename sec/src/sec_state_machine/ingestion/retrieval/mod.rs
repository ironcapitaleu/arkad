use reqwest::Error;
use retrieval_context::get_sec_user_client;
use state_maschine::prelude::*;

pub mod retrieval_context;
pub mod retrieval_data;

pub use retrieval_context::RetrievalContext;
pub use retrieval_data::RetievalData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct Retrieval {
    input: RetievalData,
    output: Option<RetievalData>,
    context: RetrievalContext,
}

impl State for Retrieval {
    type InputData = RetievalData;
    type OutputData = RetievalData;
    type Context = RetrievalContext;

    fn get_state_name(&self) -> impl ToString {
        "retrieval"
    }

    fn get_input_data(&self) -> &RetievalData {
        &self.input
    }

    fn compute_output_data(&mut self) {
        self.output = Some(RetievalData::default());
        
    }

    fn get_output_data(&self) -> Option<&RetievalData> {
        self.output.as_ref()
    }

    fn get_context_data(&self) -> &RetrievalContext {
        &self.context
    }
}

impl Retrieval {
    pub async fn compute_output_new(&self) -> Result<(), Error> {
        let cik = self.get_context_data().cik();
        let url = format!("https://data.sec.gov/api/xbrl/companyfacts/CIK{cik}.json");

        let client = get_sec_user_client()?;

        let body = client.get(&url).send().await?.text().await?;

        println!(
            "Did the retrieval process for this cik: {cik} with this body: {}...",
            &body[..100]
        );

        Ok(())
    }
}
