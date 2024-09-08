use reqwest::Error;
use retrieval_context::get_sec_user_client;
use state_maschine::prelude::*;

pub mod retrieval_context;
pub mod retrieval_data;

pub use retrieval_context::RetrievalContext;
pub use retrieval_data::RetrievalData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct Retrieval {
    input: RetrievalData,
    output: Option<RetrievalData>,
    context: RetrievalContext,
}

impl State for Retrieval {
    type InputData = RetrievalData;
    type OutputData = RetrievalData;
    type Context = RetrievalContext;

    fn get_state_name(&self) -> impl ToString {
        "retrieval"
    }

    fn get_input_data(&self) -> &RetrievalData {
        &self.input
    }

    fn compute_output_data(&mut self) {
        self.output = Some(RetrievalData::default());
    }

    fn get_output_data(&self) -> Option<&RetrievalData> {
        self.output.as_ref()
    }

    fn get_context_data(&self) -> &RetrievalContext {
        &self.context
    }
}

impl Retrieval {
    /// Computes the output by retrieving data from the SEC API.
    ///
    /// This function sends an HTTP GET request to the SEC's API using the CIK (Central Index Key)
    /// to retrieve company facts in JSON format. The result is printed out for the first 100
    /// characters.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    /// - If the HTTP client cannot be built (see [`get_sec_user_client`] for details).
    /// - If the request to the SEC API fails (e.g., network errors, invalid response).
    /// - If the body of the HTTP response cannot be retrieved or parsed.
    ///
    /// The errors are wrapped in a [`reqwest::Error`] or any custom `Error` type if applicable.
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
