use std::fmt;

use async_trait::async_trait;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::traits::state_machine::state::State;

pub mod context;
pub mod data;

pub use context::ValidateSecResponseContext;
pub use data::ValidateSecResponseInputData;
pub use data::ValidateSecResponseOutputData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateSecResponse {
    input: ValidateSecResponseInputData,
    context: ValidateSecResponseContext,
    output: Option<ValidateSecResponseOutputData>,
}

impl ValidateSecResponse {
    #[must_use]
    pub const fn new(input: ValidateSecResponseInputData, context: ValidateSecResponseContext) -> Self {
        Self {
            input,
            context,
            output: None,
        }
    }
}

#[async_trait]
impl State for ValidateSecResponse {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.output = Some(ValidateSecResponseOutputData {
            output_data: "Hello World!".to_string(),
        });
        Ok(())
    }
}

impl SMState for ValidateSecResponse {
    type InputData = ValidateSecResponseInputData;
    type OutputData = ValidateSecResponseOutputData;
    type Context = ValidateSecResponseContext;

    fn get_state_name(&self) -> impl ToString {
        "Validate SEC Response"
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

impl fmt::Display for ValidateSecResponse {
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
