mod builder;
pub mod constants;

use futures_util::StreamExt;
use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::prelude::*;
use uuid::Uuid;

use builder::{ExtractionBuilder, NoCik};

pub struct Extraction {
    raw_cik: String,
}

impl Extraction {
    pub const fn builder() -> ExtractionBuilder<NoCik> {
        ExtractionBuilder::new()
    }

    #[must_use]
    pub fn into_stream(self, execution_id: Uuid) -> StateMachineStream {
        let state = ExtractSuperState::<ValidateCikFormat>::new(self.raw_cik);
        state.into_stream(execution_id)
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cik = self.raw_cik.clone();
        let execution_id = Uuid::new_v4();
        let mut stream = std::pin::pin!(self.into_stream(execution_id));

        while let Some(result) = stream.next().await {
            match result {
                Ok(item) => {
                    println!(
                        "[{execution_id}] [{cik}] {}: '{}' | data: {}",
                        item.event, item.state_name, item.data
                    );
                }
                Err(e) => {
                    eprintln!(
                        "[{execution_id}] [{cik}] {}: '{}' | error: {}",
                        e.event, e.state_name, e.source
                    );
                    return Err(Box::new(e));
                }
            }
        }

        Ok(())
    }
}
