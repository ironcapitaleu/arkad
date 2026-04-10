mod builder;

use futures_util::StreamExt;
use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::phase_stream::PhaseStream;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;

use builder::{ExtractionBuilder, NoCik};

pub struct Extraction {
    raw_cik: String,
}

impl Extraction {
    pub const fn builder() -> ExtractionBuilder<NoCik> {
        ExtractionBuilder::new()
    }

    pub fn into_stream(self) -> PhaseStream {
        let state = ExtractSuperState::<ValidateCikFormat>::new(self.raw_cik);
        state.into_stream()
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stream = std::pin::pin!(self.into_stream());

        while let Some(result) = stream.next().await {
            let phase_output = result?;
            println!("{phase_output}");
        }

        Ok(())
    }
}
