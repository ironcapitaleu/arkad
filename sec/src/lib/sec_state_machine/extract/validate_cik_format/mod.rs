use state_maschine::prelude::*;
use std::fmt;

pub mod vcf_context;
pub mod vcf_data;

pub use vcf_context::ValidateCikFormatContext;
use vcf_data::Cik;
pub use vcf_data::ValidateCikFormatInputData;
pub use vcf_data::ValidateCikFormatOutputData;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct ValidateCikFormat {
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

    fn compute_output_data(&mut self) {
        // Validate the CIK format
        let cik = Cik::new(&self.input.raw_cik);

        // Create the output data
        let output_data = ValidateCikFormatOutputData { validated_cik: cik };

        // Set the output data
        self.output = Some(output_data);
    }

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

impl fmt::Display for ValidateCikFormat {
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
