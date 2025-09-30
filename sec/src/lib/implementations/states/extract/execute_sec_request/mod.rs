use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::traits::state_machine::state::State;
use crate::error::state_machine::state::request_execution_failed::RequestExecutionFailed;

pub mod context;
pub mod data;

pub use context::ExecuteSecRequestContext;
pub use data::ExecuteSecRequestInputData;
pub use data::ExecuteSecRequestOutputData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ExecuteSecRequest {
    input: ExecuteSecRequestInputData,
    context: ExecuteSecRequestContext,
    output: Option<ExecuteSecRequestOutputData>,
}

impl ExecuteSecRequest {
    #[must_use]
    pub const fn new(input: ExecuteSecRequestInputData, context: ExecuteSecRequestContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }
}

#[async_trait]
impl State for ExecuteSecRequest {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        let client = &self.input.sec_client;
        let request = &self.input.sec_request;

        let result = client.execute_request(request.clone()).await;

        match result {
            Ok(response) => {
                self.output = Some(ExecuteSecRequestOutputData::new(response)?);
                Ok(())
            }
            Err(e) => {

                let e: StateError = RequestExecutionFailed::new(self.get_state_name().to_string(), e).into();
                return Err(e);
            }
        }
    }
}

impl SMState for ExecuteSecRequest {
    type InputData = ExecuteSecRequestInputData;
    type OutputData = ExecuteSecRequestOutputData;
    type Context = ExecuteSecRequestContext;

    fn get_state_name(&self) -> impl ToString {
        "Execute SEC Request State"
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

impl fmt::Display for ExecuteSecRequest {
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
