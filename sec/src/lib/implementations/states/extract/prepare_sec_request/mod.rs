use std::fmt;

use async_trait::async_trait;
use reqwest::{ClientBuilder, Method, Request, Url};
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::traits::state_machine::state::State;

pub mod psr_context;
pub mod psr_data;

pub use psr_context::PrepareSecRequestContext;
pub use psr_data::PrepareSecRequestInputData;
pub use psr_data::PrepareSecRequestOutputData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct PrepareSecRequest {
    input: PrepareSecRequestInputData,
    context: PrepareSecRequestContext,
    output: Option<PrepareSecRequestOutputData>,
}

impl PrepareSecRequest {
    #[must_use]
    pub const fn new(input: PrepareSecRequestInputData, context: PrepareSecRequestContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }
}

#[async_trait]
impl State for PrepareSecRequest {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let client = ClientBuilder::new()
            .user_agent(&self.input.user_agent)
            .build();
        let Ok(client) = client else {
            return Err(StateError::ClientCreationFailed(
                "Failed to create client".to_string(),
            ));
        };

        let url_string = format!(
            "https://data.sec.gov/submissions/CIK{}.json",
            self.input.validated_cik.value()
        );

        let request = Request::new(
            Method::GET,
            Url::parse(&url_string).map_err(|_| StateError::InvalidInputData)?,
        );

        self.output = Some(
            PrepareSecRequestOutputData::new(client, request)
                .expect("Should always work since validation is done beforehand"),
        );

        Ok(())
    }
}

impl SMState for PrepareSecRequest {
    type InputData = PrepareSecRequestInputData;
    type OutputData = PrepareSecRequestOutputData;
    type Context = PrepareSecRequestContext;

    fn get_state_name(&self) -> impl ToString {
        "PrepareSecRequest State"
    }

    fn compute_output_data(&mut self) {}

    fn get_context_data(&self) -> &Self::Context {
        &self.context
    }

    fn get_input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn get_output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl fmt::Display for PrepareSecRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "`{}` State Summary\n\
             ---------------------------\n\
             Context:\n{}\n\
             Input Data:\n{}\n\
             Output Data:\n{}",
            self.get_state_name().to_string(),
            self.context,
            self.input,
            self.output.as_ref().map_or_else(
                || "\tNone".to_string(),
                |output_data| format!("{output_data}")
            )
        )
    }
}
