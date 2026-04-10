mod builder;

use futures_util::StreamExt;
use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::prelude::*;

use builder::{ExtractionBuilder, NoCik};

pub struct Extraction {
    raw_cik: String,
}

impl Extraction {
    pub const fn builder() -> ExtractionBuilder<NoCik> {
        ExtractionBuilder::new()
    }

    #[must_use]
    pub fn into_stream(self) -> StateMachineStream {
        let state = ExtractSuperState::<ValidateCikFormat>::new(self.raw_cik);
        state.into_stream()
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cik = self.raw_cik.clone();
        let mut stream = std::pin::pin!(self.into_stream());

        while let Some(result) = stream.next().await {
            let phase_output = result?;
            println!("[CIK {cik}] {phase_output}");
        }

        Ok(())
    }
}
