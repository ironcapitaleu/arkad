use state_maschine::prelude::*;

pub mod vcf_context;
pub mod vcf_data;

pub use vcf_context::ValidateCikFormatContext;
pub use vcf_data::ValidateCikFormatInputData;
pub use vcf_data::ValidateCikFormatOutputData;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
struct ValidateCikFormat {
    input: ValidateCikFormatInputData,
    context: ValidateCikFormatContext,
    output: Option<ValidateCikFormatOutputData>,
}

impl State for ValidateCikFormat {
    type InputData = ValidateCikFormatInputData;
    type OutputData = ValidateCikFormatOutputData;
    type Context = ValidateCikFormatContext;

    fn get_state_name(&self) -> impl ToString {
        "CIK Validation"
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
