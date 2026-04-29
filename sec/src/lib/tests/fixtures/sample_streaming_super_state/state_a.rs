use std::fmt::{self, Display, Formatter};

use async_trait::async_trait;
use serde::Serialize;
use state_maschine::prelude::State as SMState;

use crate::error::State as StateError;
use crate::traits::state_machine::state::State;

use super::{SampleStreamingContext, SampleStreamingData};

/// First state in the sample streaming pipeline.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord, Serialize)]
pub struct SampleStateA {
    input: SampleStreamingData,
    context: SampleStreamingContext,
    output: Option<SampleStreamingData>,
}

impl SampleStateA {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            input: SampleStreamingData,
            context: SampleStreamingContext,
            output: None,
        }
    }
}

#[async_trait]
impl State for SampleStateA {
    async fn compute_output_data_async(&mut self) -> Result<(), StateError> {
        self.output = Some(SampleStreamingData);
        Ok(())
    }
}

impl SMState for SampleStateA {
    type InputData = SampleStreamingData;
    type OutputData = SampleStreamingData;
    type Context = SampleStreamingContext;

    fn state_name(&self) -> impl ToString {
        "Sample State A"
    }

    fn compute_output_data(&mut self) {}

    fn context_data(&self) -> &Self::Context {
        &self.context
    }

    fn input_data(&self) -> &Self::InputData {
        &self.input
    }

    fn output_data(&self) -> Option<&Self::OutputData> {
        self.output.as_ref()
    }
}

impl Display for SampleStateA {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "SampleStateA")
    }
}
