use std::pin::pin;

use futures_util::StreamExt;
use sec::implementations::states::extract::ExtractSuperState;
use sec::implementations::states::extract::validate_cik_format::ValidateCikFormat;
use sec::prelude::*;
use uuid::Uuid;

/// Marker type indicating no CIK has been provided yet.
pub struct NoCik;

/// Typestate builder for constructing a [`Pipeline`].
pub struct PipelineBuilder<C> {
    cik: C,
}

impl PipelineBuilder<NoCik> {
    pub(super) const fn new() -> Self {
        Self { cik: NoCik }
    }

    pub fn cik(self, cik: impl Into<String>) -> PipelineBuilder<String> {
        PipelineBuilder { cik: cik.into() }
    }
}

impl PipelineBuilder<String> {
    pub fn build(self) -> Pipeline {
        Pipeline { raw_cik: self.cik }
    }
}

/// A full Extract + Transform pipeline for a single CIK.
pub struct Pipeline {
    raw_cik: String,
}

impl Pipeline {
    pub const fn builder() -> PipelineBuilder<NoCik> {
        PipelineBuilder::new()
    }

    pub async fn run(self) -> Result<(), StreamError> {
        let execution_id = Uuid::new_v4();
        let state = ExtractSuperState::<ValidateCikFormat>::new(self.raw_cik);
        let mut stream = pin!(state.into_stream(execution_id));

        while let Some(result) = stream.next().await {
            if let Err(e) = result {
                return Err(e);
            }
        }

        Ok(())
    }
}
